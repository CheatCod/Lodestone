use std::ops::Mul;
use std::path::Path;
use std::{fs, fs::File};
use std::collections::HashMap;
use std::io::prelude::*;
use mongodb::{IndexModel, bson::doc, sync::Client};
use redis::{aio::MultiplexedConnection, ConnectionLike, Commands};
use rocket::State;
use rocket::fairing::Result;
use crate::MyManagedState;
use crate::managers::server_instance::{ServerInstance, InstanceConfig};
use crate::util;
use crate::properties_manager::PropertiesManager;
use crate::util::db_util::mongo_schema::*;


pub struct InstanceManager {
    instance_collection : HashMap<String, ServerInstance>,
    taken_ports : Vec<u32>, 
    path : String, // must end with /
    mongodb : Client,
    redis_connection: redis::aio::MultiplexedConnection,
}

// TODO: DB IO
// TODO : should prob change parameter String to &str
impl InstanceManager {
    pub fn new(path : String, mongodb : Client, mut redis_connection: redis::aio::MultiplexedConnection) -> Result<InstanceManager, String> {
        let path_to_config = format!("{}.lodestone_config/", path);
        fs::create_dir_all(path_to_config.as_str()).map_err(|e| e.to_string())?;
        if !Path::exists(Path::new(format!("{}server.properties", path_to_config).as_str())) {
            let mut properties_file = File::create(format!("{}server.properties", path_to_config)).unwrap();
            properties_file.write_all(
        b"enable-jmx-monitoring=false\nrcon.port=25575\nenable-command-block=false\ngamemode=survival\nenable-query=false\nlevel-name=world\nmotd=AMinecraftServer\nquery.port=25565\npvp=true\ndifficulty=easy\nnetwork-compression-threshold=256\nmax-tick-time=60000\nrequire-resource-pack=false\nmax-players=20\nuse-native-transport=true\nonline-mode=true\nenable-status=true\nallow-flight=false\nvbroadcast-rcon-to-ops=true\nview-distance=10\nserver-ip=\nresource-pack-prompt=\nallow-nether=true\nserver-port=25565\nenable-rcon=false\nsync-chunk-writes=true\nop-permission-level=4\nprevent-proxy-connections=false\nhide-online-players=false\nresource-pack=\nentity-broadcast-range-percentage=100\nsimulation-distance=10\nrcon.password=\nplayer-idle-timeout=0\nforce-gamemode=false\nrate-limit=0\nhardcore=false\nwhite-list=false\nbroadcast-console-to-ops=true\nspawn-npcs=true\nspawn-animals=true\nfunction-permission-level=2\ntext-filtering-config=\nspawn-monsters=true\nenforce-whitelist=false\nresource-pack-sha1=\nspawn-protection=16\nmax-world-size=29999984\n").unwrap();
        }

        let mut instance_collection: HashMap<String, ServerInstance> = HashMap::new();

        let instances_names_u8: Vec<Vec<u8>> = redis_connection.smembers("servers:ids").unwrap(); // I sure fucking hope nothing goes wrong here
        for instance_name_u8 in instances_names_u8 {
            let instance_name = String::from_utf8(instance_name_u8).unwrap();

        }

        let database_names = mongodb
            .list_database_names(None, None).unwrap();
        for database_name in database_names.iter() {
            if database_name.contains("-") { // TODO use db filter instead
                let config = mongodb
                    .database(database_name)
                    .collection::<InstanceConfig>("config")
                    .find_one(None, None)
                    .unwrap()
                    .unwrap();
                let key = config.uuid.clone().unwrap();
                instance_collection.insert(key, ServerInstance::new(&config, format!("{}/instances/{}/", path, config.name)));
            }
        }


        Ok(InstanceManager{
            instance_collection,
            path,
            mongodb,
            redis_connection,
            taken_ports : vec![]
        })
    }
    // TODO: server.properties 
    pub async fn create_instance(&mut self, mut config : InstanceConfig, state: &State<MyManagedState>) -> Result<String, String> {
        config.name = sanitize_filename::sanitize(config.name);

        config.uuid.clone().ok_or("uuid not found")?;
        if !config.uuid.clone().unwrap().contains("-") {
            return Err("uuid format error".to_string());
        }
        if self.check_if_name_exists(&config.name) {
            return Err(format!("{} already exists as an instance", &config.name));
        }
        //check if uuid already exists in instance_collection
        if self.instance_collection.contains_key(&config.uuid.clone().unwrap()) {
            return Err(format!("{} already exists as an instance", &config.uuid.unwrap()));
        }

        
        fs::create_dir_all("tmp").map_err(|_| "couldn't create temp folder".to_string())?;
        util::download_file(&config.url.clone().unwrap(), format!("tmp/{}", &config.uuid.clone().unwrap()).as_str(), state, config.uuid.clone().unwrap().as_str()).await?; // TODO: get rid of await
        
        let path_to_instance = format!("{}instances/{}/", self.path, config.name);
        let instance = ServerInstance::new(&config, path_to_instance.clone());
        fs::create_dir_all(path_to_instance.as_str()).map_err(|e| e.to_string())?;
        fs::rename(format!("tmp/{}", &config.uuid.clone().unwrap()).as_str(), format!("{}server.jar", path_to_instance).as_str()).map_err(|_| "failed to copy file".to_string())?;
        let path_to_eula = format!("{}eula.txt", path_to_instance);
        let mut eula_file = File::create(path_to_eula.as_str()).map_err(|_|"failed to create eula.txt".to_string())?;
        eula_file.write_all(b"#generated by Lodestone\neula=true\n").map_err(|_| "failed to write to eula,txt".to_string())?;
        
        let path_to_properties = format!("{}server.properties", path_to_instance);
        self.instance_collection.insert(config.uuid.clone().unwrap(), instance);
        fs::copy(".lodestone_config/server.properties", path_to_properties).unwrap();
        match config.port {
            None => {
                for port in 25565..26000 {
                    if !self.taken_ports.contains(&port) {
                        self.taken_ports.push(port);
                        println!("using port {}", port);
                        let mut pm = PropertiesManager::new(format!("{}server.properties", path_to_instance)).unwrap();
                        pm.edit_field("server-port".to_string(), port.to_string()).unwrap();
                        pm.write_to_file().unwrap();
                        config.port = Some(port);
                        break;
                    }
                }
            }
            Some(_) => (),
        }
        // TODO: DB IO
        /* TODO: 
            create a database with the uuid name 
            create config collection 
                config is everything needed to reconstruct the config 
                store InstanceConfig into database
        */ 
        self.mongodb
            .database(&config.uuid.clone().unwrap())
            .collection("config")
            .insert_one(doc! {
                "name": &config.name,
                "version": &config.version,
                "flavour": &config.flavour,
                "port": &config.port,
                "uuid": &config.uuid.clone().unwrap(),
                "url": &config.url.unwrap(),
                "min_ram": &config.min_ram.unwrap_or(1024),
                "max_ram": &config.max_ram.unwrap_or(2048)
            }, None).unwrap();

        self.mongodb
            .database(&config.uuid.clone().unwrap())
            .create_collection("logs", None)
            .unwrap();
        
        self.mongodb
            .database(&config.uuid.clone().unwrap())
            .collection::<Log>("logs")
            .create_index(
                IndexModel::builder()
                .keys( doc! {
                    "time": -1
                })
                .build()
            , None)
            .unwrap();
        

        Ok(config.uuid.unwrap())
    }

    pub fn get_status(&mut self, uuid: String) -> Result<String, String> {
        let instance = self.instance_collection.get_mut(&uuid).ok_or("instance does not exist".to_string())?;
        Ok(instance.get_status().to_string())
    }

    // TODO: basically drop database
    pub fn delete_instance(&mut self, uuid : String) -> Result<(), String> {
        use crate::server_instance::Status;
        
        match self.instance_collection.get(&uuid).ok_or("instance does not exist".to_string())?.get_status() {
            Status::Stopped => {
                let name = self.instance_collection.get(&uuid).unwrap().name.clone();
                self.instance_collection.remove(&uuid);
                self.mongodb
                    .database(&uuid)
                    .drop(None)
                    .unwrap();
                fs::remove_dir_all(format!("instances/{}", name)).map_err(|e| e.to_string())?;
                return Ok(())
            },
            _ => return Err("instance is running".to_string())
        }
    }

    pub fn clone_instance(&mut self, uuid : String) -> Result<(), String> {
        for pair in &self.instance_collection {
            if pair.0 == &uuid {
                if self.check_if_name_exists(&format!("{}_copy", &pair.1.name)) {
                    return Err(format!("{}_copy already exists as an instance", &pair.1.name));
                }
            }
        };
        Ok(())
    }

    pub fn player_list(&self, uuid : String) -> Result<Vec<String>, String>  {
        let ins = self.instance_collection.get(&uuid).ok_or("instance does not exist".to_string())?;
        Ok(ins.get_player_list())
    }

    pub fn player_num(&self, uuid : String) -> Result<u32, String>  {
        let ins = self.instance_collection.get(&uuid).ok_or("instance does not exist".to_string())?;
        Ok(ins.get_player_num())
    }
    
    pub fn send_command(&self, uuid : String, command : String) -> Result<(), String> {
        let instance = self.instance_collection.get(&uuid).ok_or("cannot send command to instance as it does not exist".to_string())?;
        instance.send_stdin(command).map_err(|e| format!("failed to send command to instance {} : {}", instance.uuid.clone(), e))?;
        Ok(())
    }

    pub fn start_instance(&mut self, uuid : String) -> Result<(), String> {
        let instance = self.instance_collection.get_mut(&uuid).ok_or("instance cannot be started as it does not exist".to_string())?;
        instance.start(self.mongodb.clone())
    }

    pub fn stop_instance(&mut self, uuid : String) -> Result<(), String> {
        let instance = self.instance_collection.get_mut(&uuid).ok_or("instance cannot be stopped as it does not exist".to_string())?;
        instance.stop()
    }

    fn check_if_name_exists(&self, name : &String) -> bool {
        // TODO: DB IO
        let mut ret = false;
        for pair in &self.instance_collection {
            if &pair.1.name == name {
                ret = true;
                break; 
            }
        }
        ret
    }



}
