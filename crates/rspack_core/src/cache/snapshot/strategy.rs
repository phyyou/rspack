use std::{fs, path::Path, time::UNIX_EPOCH};

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub enum Strategy {
  LibVersion(String),
  UpdateTime(),
}

impl Strategy {
  pub fn lib_version<P: AsRef<Path>>(path: P) -> Option<Self> {
    // TODO add cache if performance is poor
    // TODO define StrategyGenerator
    package_json_version(path.as_ref()).map(|version| Self::LibVersion(version))
  }

  pub fn check<P: AsRef<Path>>(&self, path: P, time: u64) -> Option<bool> {
    match self {
      Self::LibVersion(version) => {
        let Some(cur_version) = package_json_version(path.as_ref()) else {
          return None;
        };
        Some(version == &cur_version)
      }
      Self::UpdateTime() => {
        let Some(file_time) = modified_time(path.as_ref()) else {
          return None;
        };
        Some(file_time < time)
      }
    }
  }
}

fn modified_time(path: &Path) -> Option<u64> {
  if let Ok(info) = fs::metadata(path) {
    if let Ok(time) = info.modified() {
      if let Ok(s) = time.duration_since(UNIX_EPOCH) {
        return Some(s.as_secs());
      }
    }
  }
  None
}

fn package_json_version(path: &Path) -> Option<String> {
  let mut cache_value = Some(path);
  while let Some(cv) = cache_value {
    if let Ok(content) = fs::read(cv.join("package.json")) {
      if let Ok(package_json) =
        serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&content)
      {
        if let Some(serde_json::Value::String(version)) = package_json.get("version") {
          return Some(version.clone());
        }
      }
      return None;
    }
    cache_value = cv.parent();
  }

  None
}
