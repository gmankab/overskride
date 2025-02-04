use std::collections::HashMap;
use gtk::glib::Sender;

use crate::{message::Message, window::ADAPTERS_LUT};

#[tokio::main]
pub async fn set_adapter_powered(adapter_name: String, sender: Sender<Message>) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;

    let current = adapter.is_powered().await?;

    adapter.set_powered(!current).await?;
    
    let powered =  adapter.is_powered().await?;
    
    if powered {
        sender.send(Message::RefreshDevicesList()).expect("can't send message");
        sender.send(Message::PopupError("br-adapter-refreshed".to_string(), adw::ToastPriority::Normal)).expect("can't send message");
    }

    sender.send(Message::SwitchAdapterPowered(powered)).expect("can't send message");
    Ok(())
}

#[tokio::main]
pub async fn set_adapter_discoverable(adapter_name: String, sender: Sender<Message>) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;
    
    let current = adapter.is_discoverable().await?;
    adapter.set_discoverable(!current).await?;

    std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
    let discoverable = adapter.is_discoverable().await?;
    sender.send(Message::SwitchAdapterDiscoverable(discoverable)).expect("can't send message");

    // println!("discoverable is: {}", discoverable);

    Ok(())
}

#[tokio::main]
pub async fn get_adapter_properties(adapters_hashmap: HashMap<String, String>, sender: Sender<Message>, adapter_name: String) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;

    let is_powered = adapter.is_powered().await?;
    let is_discoverable = adapter.is_discoverable().await?;
	let alias = adapter.alias().await?;
    let timeout = adapter.discoverable_timeout().await? / 60;
    
    sender.send(Message::PopulateAdapterExpander(adapters_hashmap)).expect("cannot send message {}");
    //println!("sent populate adapters message");
    sender.send(Message::SwitchAdapterPowered(is_powered)).expect("cannot get adapter powered.");
    sender.send(Message::SwitchAdapterDiscoverable(is_discoverable)).expect("cannot get adapter discoverable.");
    sender.send(Message::SwitchAdapterName(alias.clone().to_string(), alias.to_string())).expect("cannot get adapter name.");
    sender.send(Message::SwitchAdapterTimeout(timeout)).expect("cannot get adapter timeout.");
    
    Ok(())
}

#[tokio::main]
pub async fn set_adapter_name(alias: String, adapter_name: String, sender: Sender<Message>) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;

    let old_alias = adapter.alias().await?;
    //println!("old alias is: {}", old_alias.to_string());

    adapter.set_alias(alias.clone()).await?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    let new_alias = adapter.alias().await?;
    println!("new adapter alias is: {} compared to {}", new_alias, alias);

    unsafe {
        let mut lut = ADAPTERS_LUT.clone().unwrap();
        let bluetooth_name = adapter.name().to_string();

        lut.remove(&old_alias.clone());
        lut.insert(new_alias.clone(), bluetooth_name);
        ADAPTERS_LUT = Some(lut);
    }
    sender.send(Message::SwitchAdapterName(new_alias, old_alias)).expect("cannot change adapter name.");

    //println!("name is: {}", name.clone());
    Ok(())
}

#[tokio::main]
pub async fn set_timeout_duration(timeout: u32, adapter_name: String, sender: Sender<Message>) -> bluer::Result<()> {
    let adapter = bluer::Session::new().await?.adapter(adapter_name.as_str())?;

    adapter.set_discoverable_timeout(timeout * 60).await?;

    // std::thread::sleep(std::time::Duration::from_millis(100));
    
    let new_timeout = adapter.discoverable_timeout().await? / 60;
    sender.send(Message::SwitchAdapterTimeout(new_timeout)).expect("cannot set timeout.");

    Ok(())
}

#[tokio::main]
pub async fn populate_adapter_expander() -> bluer::Result<HashMap<String, String>> {
    let current_session = bluer::Session::new().await?;
    let adapter_names = current_session.adapter_names().await?;
    let mut alias_name_hashmap: HashMap<String, String> = HashMap::new();

    for name in adapter_names.clone() {
        let adapter = current_session.adapter(name.as_str())?;
        
       	let alias = adapter.alias().await?;
        
        alias_name_hashmap.insert(alias.clone().to_string(), name.clone().to_string());
        //println!("adapter alias is: {}", alias)
    }

    unsafe {
        ADAPTERS_LUT = Some(alias_name_hashmap.clone());
    }

    Ok(alias_name_hashmap)
}
