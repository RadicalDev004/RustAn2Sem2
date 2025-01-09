use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead};
use notify::{null, Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc, Datelike, DateTime};
use tokio::fs;
use tokio::time::{sleep, Duration};
use walkdir::WalkDir;
use async_ftp::FtpStream;
use std::sync::Arc; 
use tokio::sync::Mutex;

//INFO PARSING
#[derive(Debug)]
enum LocationType {
    FTP {user: String, password: String, url : String, path: String},
    Zip(String),
    Folder(String),
}

fn parse_location(input: &str) -> Result<LocationType, Box<dyn Error>> {
    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid format: {}", input).into());
    }
    match parts[0] {
        "ftp" => {
            let ftp_parts: Vec<&str> = parts[1].splitn(2, '@').collect();
            if ftp_parts.len() != 2 {
                return Err(format!("Invalid FTP format: {}", input).into());
            }
            let credentials: Vec<&str> = ftp_parts[0].split(':').collect();
            if credentials.len() != 2 {
                return Err(format!("Invalid FTP credentials: {}", input).into());
            }
            let path_parts: Vec<&str> = ftp_parts[1].splitn(2, '/').collect();
            if path_parts.len() != 2 {
                return Err(format!("Invalid FTP path: {}", input).into());
            }
            Ok(LocationType::FTP {
                user: credentials[0].to_string(),
                password: credentials[1].to_string(),
                url: path_parts[0].to_string(),
                path: path_parts[1].to_string(),
            })
        }
        "zip" => Ok(LocationType::Zip(parts[1].to_string())),
        "folder" => Ok(LocationType::Folder(parts[1].to_string())),
        _ => Err(format!("Unknown location type: {}", parts[0]).into()),
    }
}

fn parse_locations_from_console() -> Result<HashMap<String, LocationType>, Box<dyn Error>> {
    let stdin = io::stdin();
    let mut locations = HashMap::new();

    println!("Enter locations (type 'end' to finish):");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().eq_ignore_ascii_case("end") {
            break;
        }
        match parse_location(&line) {
            Ok(location) => {
                locations.insert(line.clone(), location);
            }
            Err(e) => {
                eprintln!("Error parsing location '{}': {}", line, e);
            }
        }
    }

    Ok(locations)
}

//LOCAL FILE SYNC
//#[derive(Debug)]
struct FileInfo {
    path: PathBuf, //full path
    modified: SystemTime,
    is_ftp: bool,
    stream: Option<Arc<Mutex<FtpStream>>>,
    ftpath: String,
}

impl FileInfo {
    pub fn debug_info(&self) -> String {
        format!(
            "FileInfo {{ path: {:?}, modified: {:?}, is_ftp: {}, ftpath: {:?} }}",
            self.path,
            self.modified,
            self.is_ftp,
            self.ftpath
        )
    }
}

struct FTP {
    stream:  Arc<Mutex<FtpStream>>,
    path: String,
    modified: SystemTime,
}

type FileTracker = HashMap<PathBuf, FileInfo>;

async fn sync_files(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::copy(src, dest).await?;
    println!("Synced file: {:?} with {:?}", dest, src);
    Ok(())
}

async fn sync_file_with_ftp(src: &FileInfo, rel: &String, dest: &Path) -> std::io::Result<()> {

    if let (Some(stream_arc), remote_path) = (&src.stream, &src.ftpath) {
        // Lock the FTP stream for safe usage
        let mut stream = stream_arc.lock().await;

        // Retrieve the file data
        match stream.simple_retr(rel).await {
            Ok(mut reader) => {                
                
                let mut local_file = fs::OpenOptions::new()
                    .write(true)
                    .create(true) // Ensures the file is created if it doesn't exist
                    .truncate(true) // Clears existing content if present
                    .open(dest.join(rel))
                    .await?;
                

                // Write the data to the destination file
                tokio::io::copy(&mut reader, &mut local_file).await?;
                println!("Synced file: {:?} from FTP path: {:?}", dest, remote_path);
            }
            Err(e) => {
                eprintln!("Failed to retrieve file from FTP: {:?}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
            }
        }
    } else {
        eprintln!("Invalid FTP stream or remote path for file: {:?}", src.path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid FTP stream or remote path",
        ));
    }

    
    Ok(())
}

async fn sync_ftp_with_file(src: &Path , dest: &FileInfo,  rel: &String) -> std::io::Result<()> {
    
    if let (Some(stream_arc), remote_path) = (&dest.stream, &dest.ftpath) {
        // Lock the FTP stream for safe usage
        let mut stream = stream_arc.lock().await;
        //println!("REEACHED HERERRR");
        
        let mut local_file = fs::OpenOptions::new()
        .read(true)
        .open(src.join(rel))
        .await?;
    
        //println!("REEACHED HERERRR");
        match stream.put(rel, &mut local_file).await {
        Ok(_) => {
            println!("Uploaded file: {:?} to FTP path: {:?}", src, dest.path);
        }
        Err(e) => {
            eprintln!("Failed to upload file to FTP: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }
    
    } else {
        eprintln!("Invalid FTP stream or remote path for file: {:?}", src);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid FTP stream or remote path",
        ));
    }

    Ok(())
}

async fn sync_ftps(src: &FileInfo, dest: &FileInfo, rel: &String) -> std::io::Result<()> {
    
let src_stream_arc = match &src.stream {
        Some(arc) => arc,
        None => {
            eprintln!("Invalid source FTP stream for file: {:?}", src.path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid source FTP stream",
            ));
        }
    };
    let mut src_stream = src_stream_arc.lock().await;


    let mut reader = match src_stream.simple_retr(rel).await {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Failed to retrieve file from source {:?} FTP: {:?}", rel, e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };


    let dest_stream_arc = match &dest.stream {
        Some(arc) => arc,
        None => {
            eprintln!("Invalid destination FTP stream for file: {:?}", dest.path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid destination FTP stream",
            ));
        }
    };
    let mut dest_stream = dest_stream_arc.lock().await;


    match dest_stream.put(rel, &mut reader).await {
        Ok(_) => {
            println!(
                "Successfully synced file: {:?} from {:?} to {:?}",
                src.path, src.ftpath, dest.ftpath
            );
        }
        Err(e) => {
            eprintln!("Failed to upload file to destination FTP: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }

    
    Ok(())
}

async fn ftp_file_exists(src: & FileInfo, filename: &str, modified: &mut SystemTime) -> std::io::Result<(bool)> {
    
    if let (Some(stream_arc), remote_path) = (&src.stream, &src.ftpath) {
        // Lock the FTP stream for safe usage
        let mut stream = stream_arc.lock().await;

        
        match stream.list(None).await {
            Ok(files) => {
                // Search for the filename in the list
                let _exists = files.iter().any(|entry| entry.contains(filename));
                if _exists {
                    
                    let file = Option::expect( files.iter().find(|&&ref x| x.contains(filename)), "AAAA");
                    let parts: Vec<&str> = file.split_whitespace().collect();
                    //let name = parts[8..].join(" ");
                    let month = parts[5];
                    let day = parts[6];
                    let time_or_year = parts[7];
                
                    let modification_time = if time_or_year.contains(':') {
                    // Parse as "Jan 05 HH:MM" with the current year
                    let current_year = Utc::now().year();
                    let date_str = format!("{} {} {} {}", current_year, month, day, time_or_year);
                    NaiveDateTime::parse_from_str(&date_str, "%Y %b %d %H:%M")
                        .ok()
                        .map(|ndt| Utc.from_utc_datetime(&ndt))
                    } else {
                    // Parse as "Jan 05 YYYY"
                        let date_str = format!("{} {} {}", month, day, time_or_year);
                        NaiveDate::parse_from_str(&date_str, "%b %d %Y")
                        .ok()
                        .and_then(|nd| nd.and_hms_opt(0, 0, 0)) // Safely create NaiveDateTime
                        .map(|ndt| Utc.from_utc_datetime(&ndt))
                    };
                    

                    *modified = SystemTime::from(Option::expect(modification_time, "WRONG FILE TIME"));
                    
                    println!("Current directory: {:?} has dir {}", stream.pwd().await, filename);
                    return Ok(true);
                }
                else {
                    println!("Didnt find {} in {:?}", filename, stream.pwd().await);
                    return Ok(false);
                }

                
            }
            Err(e) => {
                eprintln!("Failed to list files on FTP server: {:?}", e);
                return Ok(false);
            }
        }
    } else {
        eprintln!("Invalid FTP stream or remote path for file: {:?}", src.path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid FTP stream or remote path",
        ));
    }    
    
}


async fn collect_files(directory: &Path, tracker: &mut FileTracker) -> std::io::Result<()> {
    //println!("{} is the path.",  directory.display());
    for entry in WalkDir::new(directory) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let canonical_path = path.canonicalize()?;
            let relative_path= path.strip_prefix(directory).unwrap_or(path);
            if relative_path == path { continue };
            let metadata = fs::metadata(&canonical_path).await?;
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            println!("Dir {} modified {:?}", relative_path.to_string_lossy().to_string(), modified);

            tracker
                .entry(relative_path.to_path_buf())
                .and_modify(|existing| {
                    if existing.modified < modified {
                        existing.modified = modified;
                        
                        existing.path = directory.to_path_buf().clone();
                        existing.is_ftp = false;
                        existing.stream = None;
                        existing.ftpath = "".to_string();
                    }
                    
                })
                .or_insert(
                FileInfo {path: directory.to_path_buf().clone(), modified: modified.clone(), is_ftp: false, stream: None, ftpath: "".to_string()},
            );
        } else {
            println!("Skipped non-file path: {}", path.display());
        }
    }
    //println!("\nFinal tracker of all locations:\n{:#?}", tracker);
    Ok(())
}

async fn connect(url: String, username: String, password: String) -> Result<FtpStream, Box<dyn std::error::Error>> {
    println!("Connecting to the FTP server.");
    let mut ftp_stream = FtpStream::connect(url).await?;
    println!("Connected to the FTP server.");
    let _ = ftp_stream.login(&username, &password).await?;
    println!("Logged in successfully.");
    println!("Current directory: {}", ftp_stream.pwd().await?);
    
    Ok(ftp_stream)
}
async fn collect_ftp(directory: &mut FTP, tracker: &mut FileTracker) -> std::io::Result<()> {

    let mut stream = directory.stream.lock().await;
    let _ = stream.cwd(&directory.path).await;
    println!("Current working directory: {:?}", stream.pwd().await);
    
    // Access the FTP stream and list files
    let file_list = stream.list(None).await;
    
    match file_list {
        Ok(file_list) => {
            // Iterate over the file list (Vec<String>)
            for file in file_list {
                println!("Found file: {}", file);
                
                let parts: Vec<&str> = file.split_whitespace().collect();

                if parts.len() < 9 {
                    eprintln!("Unexpected listing format: {}", file);
                    continue;
                }

                let name = parts[8..].join(" ");
                let month = parts[5];
                let day = parts[6];
                let time_or_year = parts[7];
                
                let modification_time = if time_or_year.contains(':') {
                    // Parse as "Jan 05 HH:MM" with the current year
                    let current_year = Utc::now().year();
                    let date_str = format!("{} {} {} {}", current_year, month, day, time_or_year);
                    NaiveDateTime::parse_from_str(&date_str, "%Y %b %d %H:%M")
                        .ok()
                        .map(|ndt| Utc.from_utc_datetime(&ndt))
                } else {
                // Parse as "Jan 05 YYYY"
                    let date_str = format!("{} {} {}", month, day, time_or_year);
                    NaiveDate::parse_from_str(&date_str, "%b %d %Y")
                        .ok()
                        .and_then(|nd| nd.and_hms_opt(0, 0, 0)) // Safely create NaiveDateTime
                        .map(|ndt| Utc.from_utc_datetime(&ndt))
                };
                
                println!("name: {}, time: {:?}", name, modification_time);
                

                if let Some(modified) = modification_time {
                    // Convert chrono::DateTime<Utc> to SystemTime
                    let modified_system_time: SystemTime = SystemTime::from(modified);
                    directory.modified = modified_system_time;
                    
                    println!("File: {}, Modified: {:?}", name, modified_system_time);

                    tracker.entry(PathBuf::from(name.clone()))
                    .and_modify(|existing| {
                        if existing.modified < modified_system_time {
                            existing.modified = modified_system_time;
                            
                            existing.path = PathBuf::from("");
                            existing.is_ftp = true;
                            existing.stream = Some(Arc::clone(&directory.stream));
                            existing.ftpath = directory.path.clone();
                        }
                        
                    }).or_insert(FileInfo {
                        path: PathBuf::from(""),
                        modified: modified_system_time,
                        is_ftp: true,
                        stream: Some(Arc::clone(&directory.stream)),
                        ftpath: directory.path.clone(),
                    });
                } else {
                    eprintln!("Failed to parse date for file: {}", name);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to list files in {}: {}", &directory.path, e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }

    Ok(())
}

async fn sync_directories(directories: &[PathBuf], ftps: &mut [FTP]) -> std::io::Result<()> {
    let mut tracker: FileTracker = HashMap::new();

    for dir in directories {
        collect_files(dir, &mut tracker).await?;
    }
    for mut ftp in &mut *ftps {
        collect_ftp(&mut ftp, &mut tracker).await?;
    }
    
    for (path, info) in &tracker {
        println!(
            "Path: {:?}, Info: {}",
            path,
            info.debug_info()
        );
    }

    for dir in directories {
        for (relative_path, info) in &tracker {
            let dest_path = dir.join(relative_path);           
            println!("SYNCYNG FILE WITH LATEST INFO {} with {}", dest_path.to_string_lossy().to_string(), info.is_ftp);
            
            if !info.is_ftp {
                if !dest_path.exists(){
                    sync_files(&info.path.join(relative_path), &dest_path).await?;
                } else {
                    let metadata = fs::metadata(&dest_path).await?;
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    if modified < info.modified {
                        sync_files(&info.path.join(relative_path), &dest_path).await?;
                    }
                }
            }
            else {
                if !dest_path.exists(){
                    sync_file_with_ftp(&info, &relative_path.to_string_lossy().to_string(), dir).await?;
                } else {
                    let metadata = fs::metadata(&dest_path).await?;
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    println!("FTP TIME {:?} LOCAL FILE TIME {:?}", info.modified, modified);
                    if modified < info.modified {
                        sync_file_with_ftp(&info, &relative_path.to_string_lossy().to_string(), dir).await?;
                    }
                }
            }
            
        }
    }
    for ftp in ftps {
        for (relative_path, info) in &mut tracker {                    
            println!("SYNCYNG FTP WITH LATEST INFO {} at {} with {} at time {:?}", ftp.path, relative_path.to_string_lossy().to_string(), info.is_ftp, ftp.modified);
            
            let mut rel_file_modifed = SystemTime::now();
            let this_file_info = FileInfo {
                            path: PathBuf::from(""),
                            modified: ftp.modified,
                            is_ftp: true,
                            stream: Some(Arc::clone(&ftp.stream)),
                            ftpath: ftp.path.clone(),
                            };
            
            
            
            if !info.is_ftp {
 
                match ftp_file_exists(&this_file_info,&relative_path.to_string_lossy().to_string(), &mut rel_file_modifed).await
                {
                    Ok(b) => {
                        if !b {
                            println!("Syncing, file with {:?} , {} does not exist in ftp {}",info.path, relative_path.to_string_lossy().to_string(), ftp.path);
                            sync_ftp_with_file(&info.path, &this_file_info, &relative_path.to_string_lossy().to_string()).await?;
                        }
                        else {   
                            println!("Syncing, file with {:?} does {} exist in ftp {} [{:?}vs{:?}]",info.path, relative_path.to_string_lossy().to_string(), ftp.path, rel_file_modifed , info.modified);
                            if rel_file_modifed < info.modified {                                   
                                sync_ftp_with_file(&info.path, &this_file_info, &relative_path.to_string_lossy().to_string()).await?;
                            }                           
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }                         
            }
            else {
                  
                match ftp_file_exists(&this_file_info,&relative_path.to_string_lossy().to_string(), &mut rel_file_modifed).await
                {
                    Ok(b) => {                          
                        if !b {
                            println!("Syncing, ftp {} does not exist in {}", relative_path.to_string_lossy().to_string(), ftp.path);
                            sync_ftps(info,&this_file_info , &relative_path.to_string_lossy().to_string()).await?;
                        }
                        else {
                            println!("Syncing, ftp does {} exist in {} [{:?}vs{:?}]", relative_path.to_string_lossy().to_string(), ftp.path, rel_file_modifed , info.modified);
                            if rel_file_modifed < info.modified {                       
                                    sync_ftps(info,&this_file_info, &relative_path.to_string_lossy().to_string()).await?;
                            }
                        }                       
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }   
            }
            
        }
    }

    Ok(())
}

//FTP 
#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>> {   
    
    let mut all_locations = HashMap::new();
    
    loop {
        println!("Would you like to enter a new set of locations? (yes/no)");
        let mut response = String::new();
        if io::stdin().read_line(&mut response).is_err() {
            eprintln!("Error reading input.");
            continue;
        }

        if response.trim().eq_ignore_ascii_case("no") {
            break;
        }

        match parse_locations_from_console() {
            Ok(locations) => {
                all_locations.extend(locations);
                println!("Current map of all locations:\n{:#?}", all_locations);
            }
            Err(e) => eprintln!("Error reading locations: {}", e),
        }
    }

    println!("\nFinal map of all locations:\n{:#?}", all_locations);
    
    // Extract folder paths into a Vec<PathBuf>
    let folder_paths: Vec<PathBuf> = all_locations
        .values()
        .filter_map(|location| {
            if let LocationType::Folder(folder_path) = location {
                Some(PathBuf::from(folder_path))
            } else {
                None
            }
        })
        .collect();
    
    let folder_paths: Vec<PathBuf> = all_locations
        .values()
        .filter_map(|location| {
            if let LocationType::Zip(folder_path) = location {
                Some(PathBuf::from(folder_path))
            } else {
                None
            }
        })
        .collect();
    
    
    let mut ftp_paths: Vec<FTP> = Vec::new();
    for (_key, loc) in all_locations {
        if let LocationType::FTP { user, password, url, path } = loc {
            // You can now use user, password, url, and path here
            match connect(url, user, password).await {
        Ok(ftp_stream) => {
            
            ftp_paths.push(FTP{path: path.replace("\\", "/"), stream: Arc::new(Mutex::new(ftp_stream)), modified: SystemTime::now() });
        }
        Err(e) => {
            println!("Failed to connect or login: {}", e);
        }
    }
        }
    }
    
    
    
    for ftp in &ftp_paths {
    println!("FTP Path: {}", ftp.path);
    }

    println!("\nFolders to synchronize:\n{:#?}", folder_paths);
    
    loop {                
        println!("Syncing again...");
        if folder_paths.len() + ftp_paths.len() < 2 {
            println!("Not enough folders to synchronize.");
            break;
        }
        if !folder_paths.is_empty() || !ftp_paths.is_empty() {
            if let Err(e) = sync_directories(&folder_paths,&mut ftp_paths).await {
                eprintln!("Error during synchronization: {}", e);
            }
        } else {
            println!("No valid folders to synchronize.");
        }
        sleep(Duration::from_secs(5)).await;
    }
    
    Ok(())
}
