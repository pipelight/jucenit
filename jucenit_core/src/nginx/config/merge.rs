use super::Config;
// Error handling
use miette::Result;

impl Config {
    pub fn merge(old: &Config, new: &Config) -> Result<Config> {
        let mut merged = old.to_owned();
        for (key, value) in new.listeners.iter() {
            // merge new listeners to old
            merged.listeners.insert(key.to_owned(), value.to_owned());

            // merge routes based on uniq match
            // let route_name = format!("jucenit_[{}]", key);
            // let mut new_routes = new.routes.get(&route_name).unwrap().clone();
        }
        for (key, value) in new.routes.iter() {
            // If route already exists then fuse and dedup
            if let Some(route) = merged.routes.get_mut(key) {
                route.extend(value.to_owned());
                route.sort_by_key(|p| p.clone().match_);
                route.dedup_by_key(|p| p.clone().match_);
            } else {
                merged.routes.insert(key.to_owned(), value.to_owned());
            }
        }
        Ok(merged)
    }
    pub fn unmerge(old: &Config, new: &Config) -> Result<Config> {
        let mut unmerged = old.to_owned();
        for (key, old_route) in unmerged.routes.iter_mut() {
            if let Some(new_route) = new.routes.get(key) {
                for step in new_route.clone() {
                    if let Some(index) = old_route.iter().position(|e| e == &step) {
                        old_route.remove(index);
                    }
                }
            }
        }
        Ok(unmerged)
    }
}

mod tests {
    use super::Config;
    use crate::cast::Config as ConfigFile;
    // Error handling
    use miette::Result;

    #[tokio::test]
    async fn merge_file_w_duplicates_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.test.com.toml")?;
        let new = Config::from(&config_file);
        let old = Config::get().await?;
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn merge_file_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.example.com.toml")?;
        let new = Config::from(&config_file);
        let old = Config::get().await?;
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn merge_config_chunks() -> Result<()> {
        let old = Config::from(&ConfigFile::from_toml("../examples/jucenit.test.com.toml")?);
        let new = Config::from(&ConfigFile::from_toml(
            "../examples/jucenit.example.com.toml",
        )?);
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn unmerge_config_chunks() -> Result<()> {
        let old = Config::from(&ConfigFile::from_toml("../examples/jucenit.full.toml")?);
        let new = Config::from(&ConfigFile::from_toml("../examples/jucenit.test.com.toml")?);
        let res = Config::unmerge(&old, &new)?;
        println!("{:#?}", res);
        Ok(())
    }
}
