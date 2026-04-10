// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde_json::from_str;
use tauri::utils::config;
use tauri::webview::cookie::time::format_description::modifier::UnixTimestamp;
use tauri::{AppHandle, Emitter};

use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use std::{thread, time};

use std::thread::{JoinHandle, spawn};
use std::path::{Path, PathBuf};

use std::fs::{self, FileTimes, read_to_string};
use std::env;

static one_sec: time::Duration = time::Duration::from_secs(1);

use rfd::FileDialog;

#[tauri::command]
fn perform_backup(app: AppHandle){

    let app_handle = app.clone();

    let thread_fn = move || {

        //get exe dir 
        let exe_path = env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();


        //get config
        let config_path = exe_dir.join("backup.conf");
        let config: String = read_to_string(&config_path).unwrap();
        let config_lines: Vec<&str> = config.lines().collect();

        if config_lines[0] == "none" {
            app_handle.emit("my_event", "kein valider Quell-Pfad ausgewählt").unwrap();
            return
        }

        if config_lines[1] == "none" {
            app_handle.emit("my_event", "kein valider Ziel-Pfad ausgewählt").unwrap();
            return
        }


        //get timestamps.json, if it doesnt exist, create an empty one
        let timestamps_str: String = match fs::read_to_string(exe_dir.join("timestamps.json")) {
            Ok(val) => val,
            Err(err) => {
                fs::File::create(exe_dir.join("timestamps.json"));
                String::from("none")
            }
        };


        let timestamp_str = fs::read_to_string(exe_dir.join("timestamps.json")).unwrap();
        let timestamps_deser = serde_json::from_str::<HashMap<PathBuf, u64>>(&timestamp_str).unwrap();




        
        let src_vector = file_walk(config_lines[0]).unwrap().0;
        let src_set: HashSet<PathBuf> = src_vector.into_iter().map(|x| x.strip_prefix(config_lines[0]).unwrap().to_path_buf()).collect();

        let dst_vector = file_walk(config_lines[1]).unwrap().0;
        let dst_set: HashSet<PathBuf> = dst_vector.into_iter().map(|x| x.strip_prefix(config_lines[1]).unwrap().to_path_buf()).collect();

        let missing_in_dst: HashSet<&PathBuf> = src_set.difference(&dst_set).collect();
        let obsolete_in_dst: HashSet<&PathBuf> = dst_set.difference(&src_set).collect();
        let contained_in_both: HashSet<&PathBuf> = src_set.intersection(&dst_set).collect();

        let mut to_be_updated: Vec<PathBuf> = Vec::new();
        for ele in contained_in_both {

            let whole_path = PathBuf::from(config_lines[0]).join(ele);
            let current_time = whole_path.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let recorded_time = timestamps_deser.get(&whole_path).unwrap_or(&0);

            if &current_time != recorded_time {
                to_be_updated.push(whole_path);
            }

        }


        if missing_in_dst.is_empty() && obsolete_in_dst.is_empty() && to_be_updated.is_empty() {
            
            let sending_string = format!("<b style=\"color: white;\">Keine neuen oder geänderten Dateien gefunden</b>");
            app_handle.emit("my_event", sending_string).unwrap();

        }


        if !missing_in_dst.is_empty() {

            let sending_string = format!("<p style=\"color: white;\">Dateien die dem Backup neu hinzugefügt wurden: </p>");
            app_handle.emit("my_event", sending_string).unwrap();

            for ele in missing_in_dst {
            let in_dst = PathBuf::from(config_lines[1]).join(ele);
            let in_src = PathBuf::from(config_lines[0]).join(ele);

            fs::create_dir_all(in_dst.parent().unwrap()).unwrap();

            fs::copy(&in_src, &in_dst).unwrap();

            let sending_string = format!("> <b style=\"color: green;\">{}</b> <br>", &in_dst.file_name().unwrap().display());
            app_handle.emit("my_event", sending_string).unwrap();


        }

        }

        if !obsolete_in_dst.is_empty(){

            let sending_string = format!("<p style=\"color: white;\">Dateien die dem Backup entfernt wurden: </p>");
            app_handle.emit("my_event", sending_string).unwrap();

            for ele in obsolete_in_dst{
                let in_dst = PathBuf::from(config_lines[1]).join(ele);

                println!("{:?}", in_dst);

                fs::remove_file(&in_dst).expect("couldnt delete");

                let sending_string = format!("> <b style=\"color: red;\">{}</b> <br>", &in_dst.file_name().unwrap().display());
                app_handle.emit("my_event", sending_string).unwrap();
            }

        }

        if !to_be_updated.is_empty(){
            let sending_string = format!("<p style=\"color: green;\">Dateien die geupdated werden: </p>");
            app_handle.emit("my_event", sending_string).unwrap();

            for ele in to_be_updated {

                let tmp = ele.strip_prefix(config_lines[0]).unwrap();
                let dst_path = PathBuf::from(config_lines[1]).join(tmp);

        
                fs::copy(&ele, dst_path);

                let sending_string = format!("> <b style=\"color: green;\">{}</b> <br>", &ele.file_name().unwrap().display());
                app_handle.emit("my_event", sending_string).unwrap();
            }

        }



        //DELETE EMTPY DIRS
        let empty_dirs = file_walk(config_lines[1]).unwrap().1;
        empty_dirs.into_iter().for_each(|x| fs::remove_dir(x).unwrap());



        //WRITE TIMESTAMPS
        let src_paths = file_walk(config_lines[0]).unwrap().0;

        let mut time_map: HashMap<PathBuf, u64> = HashMap::new();

        for ele in src_paths {

            let secs: u64 = ele.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
            time_map.insert(ele.clone(), secs);

        }

        let json = serde_json::to_string_pretty(&time_map).unwrap();
        fs::write(exe_dir.join("timestamps.json"), json);


        //SEND FINAL MESSAGE
        let sending_string = format!("<br><b style=\"color: yellow;\">Backup Ausgeführt</b>");
        app_handle.emit("my_event", sending_string).unwrap();

    };


    spawn(thread_fn);


}


#[tauri::command]
fn pick_folders(app: AppHandle, fld: String) -> String {
    let app_handle = app.clone();

    let thread_fn = move || {

        let config_path = env::current_exe().unwrap().parent().unwrap().join("backup.conf");

        

        if !config_path.exists() {


            fs::write(&config_path, "none\nnone\n").expect("couldn't write new config file");
        }

        let folder = FileDialog::new()
        /* .add_filter("Rust", &["rs"]) */
        .pick_folder();


        let selected_folder = match folder {
            Some(path) => path,
            None => return()
        };

        println!("{:?}", selected_folder);

        
        let x = read_to_string(&config_path).unwrap();
        let y: Vec<&str> = x.lines().collect();

        println!("{}", y[0]);



        if fld == "src" {
            let line = format!("{}\n{}\n", selected_folder.to_str().unwrap(), y[1]);
            fs::write(&config_path, line).expect("couldn't write to config file");

            let sending_string = format!("Quell-Ordner erfolgreich gesetzt auf: <br> <p style=\"color: green;\">{}</p>", &selected_folder.display());
            app_handle.emit("my_event", sending_string).unwrap();

        }
        else if fld == "dst" {
            let line = format!("{}\n{}\n", y[0], selected_folder.to_str().unwrap());
            fs::write(&config_path, line).expect("couldn't write to config file");

            let sending_string = format!("Ziel-Ordner erfolgreich gesetzt auf: <br> <p style=\"color: green;\">{}</p>", &selected_folder.display());
            app_handle.emit("my_event", sending_string).unwrap();

        };

    
    };


    spawn(thread_fn);

    format!("success")
}




#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![pick_folders, perform_backup])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



fn file_walk(path: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>), std::io::Error>{


    let mut dirs: Vec<PathBuf> = Vec::new();

    let mut files: Vec<PathBuf> = Vec::new();
    let mut empty_dirs: Vec<PathBuf> = Vec::new();

    dirs.push(PathBuf::from(path));

    loop {
        
        //break if no elements are left
        let current = match dirs.pop() {
            Some(val) => val,
            None => { break ;}
        };

        let iterator = fs::read_dir(current)?;

        for elem in iterator {

            let elem = elem?;

            if elem.file_type()?.is_file() {
                files.push(elem.path());
            }

            else if elem.file_type()?.is_dir() && elem.path().read_dir()?.next().is_none() {
                empty_dirs.push(elem.path());
            }

            else if elem.file_type()?.is_dir() {
                dirs.push(elem.path());
            }
        }

    }


    Ok((files, empty_dirs))

}