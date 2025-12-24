use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::model::Project;
use uuid::Uuid;
use chrono::Local;

pub struct ProjectManager;

impl ProjectManager {
    fn get_config_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from("config")
        }
    }

    fn get_data_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
            proj_dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from("data")
        }
    }

    fn get_manifest_path() -> PathBuf {
        Self::get_config_dir().join("projects.json")
    }

    pub fn load_projects() -> Vec<Project> {
        let path = Self::get_manifest_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(projects) = serde_json::from_str::<Vec<Project>>(&content) {
                    let mut sorted = projects;
                    sorted.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
                    return sorted;
                }
            }
        }

        let default_db = Self::get_data_dir().join("tas.db");
        if default_db.exists() {
            let p = Project {
                id: Uuid::new_v4().to_string(),
                name: "Arhelis (Default)".to_string(),
                path: default_db.to_string_lossy().to_string(),
                last_opened: Local::now(),
                created_at: Local::now(),
            };
            let _ = Self::save_projects(&[p.clone()]);
            return vec![p];
        }

        Vec::new()
    }

    pub fn save_projects(projects: &[Project]) -> Result<(), String> {
        let dir = Self::get_config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(projects).map_err(|e| e.to_string())?;
        fs::write(Self::get_manifest_path(), json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn create_project(name: String) -> Result<Project, String> {
        let data_dir = Self::get_data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let safe_name: String = name.chars()
            .map(|x| if x.is_alphanumeric() { x } else { '_' })
            .collect();

        let filename = format!("tas_{}_{}.db", safe_name, Uuid::new_v4().simple().to_string().get(0..6).unwrap_or("000"));
        let db_path = data_dir.join(filename);

        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            path: db_path.to_string_lossy().to_string(),
            last_opened: Local::now(),
            created_at: Local::now(),
        };

        let mut projects = Self::load_projects();
        projects.insert(0, project.clone());
        Self::save_projects(&projects)?;

        Ok(project)
    }

    pub fn update_last_opened(id: &str) {
        let mut projects = Self::load_projects();
        if let Some(p) = projects.iter_mut().find(|p| p.id == id) {
            p.last_opened = Local::now();
        }
        let _ = Self::save_projects(&projects);
    }

    // NUEVO: Función para borrar proyecto
    pub fn delete_project(id: &str) -> Result<(), String> {
        let mut projects = Self::load_projects();

        // 1. Encontrar el proyecto para obtener el path
        if let Some(pos) = projects.iter().position(|p| p.id == id) {
            let p = &projects[pos];
            let path = PathBuf::from(&p.path);

            // 2. Intentar borrar el archivo físico (si existe)
            if path.exists() {
                let _ = fs::remove_file(path); // Ignoramos error si está en uso por ahora
            }

            // 3. Eliminar de la lista
            projects.remove(pos);
            Self::save_projects(&projects)?;
        }
        Ok(())
    }
}