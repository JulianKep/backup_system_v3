// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::utils::config;
use tauri::{AppHandle, Emitter};

use std::{thread, time};

use std::thread::{JoinHandle, spawn};
use std::path::{Path, PathBuf};

use std::fs::{self, read_to_string};
use std::env;

static one_sec: time::Duration = time::Duration::from_secs(1);

use rfd::FileDialog;






#[tauri::command]
fn perform_backup(app: AppHandle){

    let app_handle = app.clone();

    let thread_fn = move || {
        let sending_string = format!("{}<br>", "hallo haha");
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
            None => PathBuf::from("none")
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



fn file_walk(path: &Path) -> Result<(), std::io::Error>{

    let mut files: Vec<PathBuf> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut empty_dirs: Vec<PathBuf> = Vec::new();

    dirs.push(path.to_path_buf());

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

    println!("{:?}", files);
    println!("{:?}", empty_dirs);


    Ok(())

}