use tide_websockets::{WebSocket, WebSocketConnection};
use std::collections::{HashMap, VecDeque};
use tide::{Request,Response,StatusCode};
use std::time::{Duration, Instant};
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use serde_json::{Value,to_string};
use serde::{Deserialize};
use std::error::Error;
use tide::prelude::*;
use std::rc::Rc;
use base64; // Import the functions you need


#[derive(Clone)]
pub struct TideIO {
    engineio : Arc<EngineIO>
}
impl TideIO{
    pub fn new() -> Self {
        Self {
            engineio : Arc::new(EngineIO::new()),
        }
    }
    // Function to add a listener (for testing purposes)
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(Vec<Value>,String, String) + Send + 'static,
    {
        self.engineio.on(event,callback);
    }

    pub fn emit(&self, event: &str, data: Vec<Value>, namespace: String, sid: String)
    {
        self.engineio.emit(event, data, namespace, sid);
    }

    pub async fn handle(&self, req: tide::Request<()>) -> tide::Result<Response>
    {
        self.engineio.handle_engine_io(req).await
    }

}

#[derive(Deserialize)]
#[serde(default)]
struct Query {
    EIO: u32,
    transport: String,
    t: String,
    sid: String
}
impl Default for Query {
    fn default() -> Self {
        Self {
            EIO: 0,
            transport: "".to_string(),
            t: "".to_string(),
            sid: "".to_string()
        }
    }
}

const IO_CONNECT: &str = "0";
const IO_DISCONNECT: &str = "1";
const IO_EVENT: &str = "2";
const IO_ACK: &str = "3";
const IO_CONNECT_ERROR: &str = "4";
const IO_BINARY_EVENT: &str = "5";
const IO_BINARY_ACK: &str = "6";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Packet {
    CONNECT {
        id: String,
        namespace: String,
        data: Value, // Use `Option<Value>` for optional data payload
    },
    DISCONNECT {
        id: String,
        namespace: String,
        data: Value, // Use `Option<Value>` for optional data payload
    },
    CONNECT_ERROR {
        id: String,
        namespace: String,
        data: Value,
    },
    EVENT {
        id: String,
        namespace: String,
        data: Vec<Value>, // `data` can be an array of values
    },
    ACK {
        id: String,
        namespace: String,
        data: Value,
    },
    BINARY_EVENT {
        id: String,
        namespace: String,
        data: Vec<Value>, // You will need to handle binary separately
    },
    BINARY_ACK {
        id: String,
        namespace: String,
        data: Vec<Value>, // You will need to handle binary separately
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryData {
    _placeholder: bool,
    num: u32,
}

const BUFFER_START : &str = "<Buffer <";
const BUFFER_END : &str = ">>";
fn parse_buffers(buffer_str: &str) -> Result<Vec<u8>, tide::Error>{
    let buffer_str = buffer_str.trim();
    if buffer_str.starts_with(BUFFER_START) && buffer_str.ends_with(BUFFER_END) {
        let hex_data = &buffer_str[BUFFER_START.len()..(buffer_str.len() - BUFFER_END.len())].trim();
        let hex_bytes: Result<Vec<u8>, _> = hex_data
            .split_whitespace()
            .map(|hex| u8::from_str_radix(hex,16))
            .collect();

        // println!("hex_bytes: {}", hex_bytes);

        hex_bytes.map_err(|_| tide::Error::from_str(400, "Failed to parse buffer hex data"))
    } else {
        Err(tide::Error::from_str(400, "Invalid buffer format"))  // Return tide::Error for invalid format
    }
}

fn replace_placeholders(value: &mut serde_json::Value, buffers: Vec<Vec<u8>>) -> Result<(), &'static str> {
    // Parse the JSON part
    match value {
        serde_json::Value::Array(ref mut arr) => {
            for v in arr.iter_mut()
            {
                replace_placeholders(v,buffers.clone())?;
            }
        }
        serde_json::Value::Object(ref mut map) => {
            if let Some(serde_json::Value::Bool(true)) = map.get("_placeholder") {
                if let Some(serde_json::Value::Number(num)) = map.get("num") {
                    let index = num.as_u64().ok_or("Invalid buffer index")? as usize;
                    if index < buffers.len() {
                        let buffer = &buffers[index];
                        *value = serde_json::Value::String(base64::encode(buffer));
                    } else {
                        return Err("Buffer index out of range");
                    }
                }
            }
        },
        _ => {}
    }
    return Ok(());
}

fn encode_packet(packet: Packet) -> tide::Result<String> {
    match packet {
        Packet::CONNECT { namespace, data, id } => {println!("Encoding Not Iimplemented yet");},
        Packet::DISCONNECT { namespace, data, id } => {println!("Encoding Not Iimplemented yet");},
        Packet::EVENT { namespace, data, id } => {
            let mut payload = String::from("42");
            if namespace != String::from("/") {
                payload = payload + &namespace + ",";
            }

            if id != String::from("") {
                payload = payload + &id;
            }

            let json_array = Value::Array(data);
            let json_string = to_string(&json_array).unwrap();

            payload += &json_string;
            return Ok(payload.to_string());
        },
        Packet::ACK { namespace, data, id } => {println!("Encoding Not Iimplemented yet");},
        Packet::CONNECT_ERROR { namespace, data, id } => {println!("Encoding Not Iimplemented yet");},
        Packet::BINARY_EVENT { namespace, data, id } => {
                // let mut payload = String::from("45");
            
                // let binary_attachments = mixed_data.len() - 1;
                // if binary_attachments > 0 {
                //     payload = payload + &binary_attachments + "-";
                // }
                
                // if namespace != String::from("/") {
                //     payload = payload + &namespace + ",";
                // }

                // if id != String::from("") {
                //     payload = payload + &id;
                // }


                // let json_array = Value::Array(data);
                // let json_string = to_string(&json_array).unwrap();
    
                // payload += &json_string;
                // return Ok(payload.to_string());
                println!("Encoding Not Iimplemented yet");
        },
        Packet::BINARY_ACK { namespace, data, id } => {println!("Encoding Not Iimplemented yet");},//handle_noop(&payload)},
        // Handle other packet types as needed
        _ => {println!("Unknown packet to encode");},
    }
    Ok(String::from("Not Implemented Yet"))
} 

fn decode_packet(encoded: &str) -> tide::Result<Packet> {
    // Split the encoded packet into namespace and payload parts
    if  encoded.starts_with(IO_CONNECT) ||
        encoded.starts_with(IO_DISCONNECT) ||
        encoded.starts_with(IO_EVENT) ||
        encoded.starts_with(IO_ACK) ||
        encoded.starts_with(IO_CONNECT_ERROR){

        let message_type = &encoded[0..1];
        let message_type_idx = 1;
        
        let mut sub_encoded = &encoded[message_type_idx..];
        let (namespace, namespace_end_idx) = match sub_encoded.find(',') {
            Some(comma_idx) => {
                // Find positions of `{` and `[`
                let brace_pos = sub_encoded.find('{');
                let bracket_pos = sub_encoded.find('[');
        
                // Check if the comma is before both `{` and `[` if they exist
                if brace_pos.map_or(true, |pos| comma_idx < pos) && bracket_pos.map_or(true, |pos| comma_idx < pos) {
                    (&sub_encoded[..comma_idx], comma_idx+1)
                } else {
                    ("/", 0) // If the comma is after `{` or `[`, treat it as no namespace
                }
            },
            None => ("/", 0)
        };

        let mut sub_encoded = &sub_encoded[namespace_end_idx..];
        let (id, id_end_idx) = match sub_encoded.find('[') {
            Some(bracket_pos) => (&sub_encoded[..bracket_pos], bracket_pos),
            None => ("", 0)
        };

        let payload = &encoded[(message_type_idx+namespace_end_idx + id_end_idx)..];
            
        let first_char_string = encoded.chars().next().map_or_else(|| "".to_string(), |c| c.to_string());
        match first_char_string.as_str() {
            IO_CONNECT => {
                let json_payload: Value = match serde_json::from_str(payload){
                    Ok(value) => value,
                    Err(_) => Value::Object(serde_json::Map::new()), 
                };
                Ok(Packet::CONNECT {
                    id: id.to_string(),
                    namespace: namespace.to_string(),
                    data: json_payload
                })
            },
            IO_DISCONNECT => {
                let json_payload: Value = match serde_json::from_str(payload){
                    Ok(value) => value,
                    Err(_) =>Value::Object(serde_json::Map::new()), 
                };
                Ok(Packet::DISCONNECT {
                    id: id.to_string(),
                    namespace: namespace.to_string(),
                    data: json_payload
                })
            },
            IO_EVENT => {
                // TODO: this is parsed wrongly
                let json_payload: Value = match serde_json::from_str(payload){
                    Ok(value) => value,
                    Err(_) =>Value::Object(serde_json::Map::new()), 
                };
                Ok(Packet::EVENT {
                    id: id.to_string(),
                    namespace: namespace.to_string(),
                    data: json_payload.as_array().unwrap_or(&vec![]).clone()
                })
            },
            IO_ACK => {
                let json_payload: Value = match serde_json::from_str(payload){
                    Ok(value) => value,
                    Err(_) =>Value::Object(serde_json::Map::new()), 
                };
                Ok(Packet::ACK {
                    id: id.to_string(),
                    namespace: namespace.to_string(),
                    data: json_payload
                })
            },
            IO_CONNECT_ERROR => {
                let json_payload: Value = match serde_json::from_str(payload){
                    Ok(value) => value,
                    Err(_) =>Value::Object(serde_json::Map::new()), 
                };
                Ok(Packet::CONNECT_ERROR {
                    id: id.to_string(),
                    namespace: namespace.to_string(),
                    data: json_payload
                })
            },
            &_ => {
                Err(tide::Error::from_str(StatusCode::NotFound, "Wrongly jsonized"))
            }
        }

    } else if encoded.starts_with(IO_BINARY_EVENT) ||
              encoded.starts_with(IO_BINARY_ACK) {
        let (message_type, binary_attachments, end_indx) = match encoded.find('-') {
            Some(dash_indx) => (&encoded[0..1], &encoded[1..dash_indx], dash_indx+1),
            None => (&encoded[0..1], "", 1)
        };
        
        let mut sub_encoded = &encoded[end_indx..];
        let (namespace, namespace_end_idx) = match sub_encoded.find(',') {
            Some(comma_idx) => {
                // Find positions of `{` and `[`
                let brace_pos = sub_encoded.find('{');
                let bracket_pos = sub_encoded.find('[');
        
                // Check if the comma is before both `{` and `[` if they exist
                if brace_pos.map_or(true, |pos| comma_idx < pos) && bracket_pos.map_or(true, |pos| comma_idx < pos) {
                    (&sub_encoded[..comma_idx], comma_idx+1)
                } else {
                    ("/", 0) // If the comma is after `{` or `[`, treat it as no namespace
                }
            },
            None => ("/", 0)
        };

        let mut sub_encoded = &sub_encoded[namespace_end_idx..];
        let (id, id_end_idx) = match sub_encoded.find('[') {
            Some(bracket_pos) => (&sub_encoded[..bracket_pos], bracket_pos),
            None => ("", 0)
        };

        let raw_payload = &encoded[(end_indx+namespace_end_idx + id_end_idx)..];
        
        let parts: Vec<&str> = raw_payload.split('+').collect();
        let mut payload = "";
        let buffers : Vec<Vec<u8>>; // can I initialize it without value
        if parts.is_empty(){
            payload = raw_payload;
            Ok(Packet::BINARY_EVENT{
                id : id.parse().unwrap(),
                namespace: namespace.to_string(),
                data: vec![]
            })
        }
        else
        {
            // parse bufferrs
            payload = parts[0];
            buffers = parts[1..].iter()
            .map(|buffer_str| parse_buffers(buffer_str.trim()))
            .collect::<Result<Vec<Vec<u8>>,_>>()?;

            let mut parsed: serde_json::Value = serde_json::from_str(payload)?;
            replace_placeholders(&mut parsed, buffers);

            return match parsed {
                Value::Array(arr) => 
                    Ok(Packet::BINARY_EVENT{
                        id: id.to_string(),
                        namespace: namespace.to_string(),
                        data: arr
                    }),  // If it's an array, return it
                _ => Err(tide::Error::from_str(StatusCode::NotFound, "Parsed data is not array")),  // If not, return an error
            };
        }
    } else {
        Err(tide::Error::from_str(StatusCode::NotFound, "Wrongly jsonized"))
    }
}

// Define a struct to store the items in the queue
#[derive(Debug)]
struct QueuedEvent {
    event: String,
    data: String,
    sid : String,
}

// Define a type alias for the queue
type SharedQueue = Arc<Mutex<HashMap<String,VecDeque<QueuedEvent>>>>;

#[derive(Clone)]
pub struct SocketIO {
    eventQueues : SharedQueue,
    listeners  : Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(Vec<Value>,String, String) + Send>>>>>,
    namespaces : Arc<Mutex<HashMap<String, Vec<String>>>>,
    sid2namespaces : Arc<Mutex<HashMap<String, String>>>,
    sidconncted : Arc<Mutex<HashMap<String, bool>>>,
}
impl SocketIO{
    pub fn new() -> Self {
        Self {
            eventQueues : Arc::new(Mutex::new(HashMap::new())),
            listeners  : Arc::new(Mutex::new(HashMap::new())),
            namespaces : Arc::new(Mutex::new(HashMap::new())),
            sid2namespaces : Arc::new(Mutex::new(HashMap::new())),
            sidconncted : Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Function to add a listener (for testing purposes)
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(Vec<Value>,String, String) + Send + 'static,
    {
        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    pub fn emit(&self, event: &str, data: Vec<Value>, namespace: String, sid: String)
    {
        let mut payload = data.clone();
        payload.insert(0, Value::String(event.to_string()));
        
        let packet = Packet::EVENT{
            id : String::from(""),
            namespace : namespace.to_string(),
            data : payload,
        };

        let packet_encoded = encode_packet(packet);

        let QueuedEvent = QueuedEvent{
            event : event.clone().to_string(),
            data : packet_encoded.expect("REASON").clone().to_string(),
            sid : sid.clone().to_string(),
        };    
        println!("trying to send {} to queue:{}",event,sid);
        
        if self.is_connected(sid.as_str()) {
            println!("trying to find queue:{}",sid);
            let mut queues = self.eventQueues.lock().unwrap();
            if let Some(q) = queues.get_mut(sid.as_str()) {
                println!("sending {} to queue:{}",event,sid);
                q.push_back(QueuedEvent);
            }
            else {
                queues.insert(sid.to_string(),VecDeque::new());
                if let Some(q) = queues.get_mut(sid.as_str()) {
                    println!("sending {} to queue:{}",event,sid);
                    q.push_back(QueuedEvent);
                }
            }
        }
    }

    pub fn get_payload(&self,sid: &str) -> Option<QueuedEvent> // TODO: return QueuedEvent or error 
    {
        if self.is_connected(sid) {
            let mut queues = self.eventQueues.lock().unwrap();
            if let Some(q) = queues.get_mut(sid) {
                return q.pop_front();
            }
        }
        // let mut q : std::sync::MutexGuard<'_, VecDeque<QueuedEvent>> = self.eventQueue.lock().unwrap();
        // q.pop_front()
        None
    }

    fn set_connect(& self, sid: &str){
        let mut sidconncted = self.sidconncted.lock().unwrap();
        sidconncted.insert(sid.to_string(),true);
    }

    fn set_disconnect(& self, sid: &str){
        let mut sidconncted = self.sidconncted.lock().unwrap();
        sidconncted.insert(sid.to_string(),false);
    }

    pub fn is_connected(& self, sid: &str) -> bool {
        let sidconncted = self.sidconncted.lock().unwrap();
        *sidconncted.get(sid).unwrap_or(&false)
    }


    pub fn is_registered_namespace(& self, sid: &str) -> bool {
        let sid2namespaces = self.sid2namespaces.lock().unwrap();
        let contains = sid2namespaces.contains_key(sid);
        return contains;
    }

    // Function to insert a value into the namespaces map
    pub fn insert_namespace(&self, namespace: String, value: String) {
        let mut namespaces = self.namespaces.lock().unwrap();
        namespaces.entry(namespace.clone()).or_insert_with(Vec::new).push(value.clone());
        let mut sid2namespaces = self.sid2namespaces.lock().unwrap();
        sid2namespaces.insert(value,namespace);
    }

    // Function to remove a key from the namespaces map
    pub fn remove_namespace(&self, namespace: &str, sid: &str) -> Option<String> {
        self.remove_value_from_namespace(namespace,sid);
        let mut sid2namespaces = self.sid2namespaces.lock().unwrap();
        sid2namespaces.remove(sid) // Removes the key and returns the associated value, if any
    }

    // Function to remove a specific value from the vector in the namespaces map
    pub fn remove_value_from_namespace(&self, namespace: &str, value: &str) {
        let mut namespaces = self.namespaces.lock().unwrap();
        if let Some(values) = namespaces.get_mut(namespace) {
            if let Some(pos) = values.iter().position(|x| x == value) {
                values.remove(pos);
            }
        }
    }

    fn handle_connect(& self, namespace: &str, payload: Value, sid: &str) -> tide::Result<Response> {
        println!("handling connect namespace {}", namespace);
        println!("handling connect payload {:?}", payload);
        
        self.insert_namespace(
            namespace.to_string(),
            sid.to_string()
        );
        let response_json = json!({"sid" : sid.to_string()});
        
        let mut response = Response::new(200);
        if namespace == "/" {
            let response_body = format!("40{}", serde_json::to_string(&response_json)?);
            response.set_body(response_body);
        }
        else{
            let response_body = format!("40{},{}", namespace, serde_json::to_string(&response_json)?);
            response.set_body(response_body);
        }
        // response.insert_header("Content-Type", "application/json");
        response.insert_header("Content-Type", "text/plain; charset=UTF-8");
        Ok(response)
    }

    fn handle_disconnect(& self, namespace: &str, payload: Value, sid: &str) -> tide::Result<Response> {
        println!("handling connect namespace {}", namespace);
        println!("handling connect payload {:?}", payload);
        
        self.remove_namespace(
            namespace,
            sid
        );
        self.set_disconnect(sid);

        let response_json = json!({"sid" : sid.to_string()});
        let mut response = Response::new(200);
        if namespace == "/" {
            let response_body = format!("41{}", serde_json::to_string(&response_json)?);
            response.set_body(response_body);
        }
        else{
            let response_body = format!("41{},{}", namespace, serde_json::to_string(&response_json)?);
            response.set_body(response_body);
        }
        // response.insert_header("Content-Type", "application/json");
        response.insert_header("Content-Type", "text/plain; charset=UTF-8");
        Ok(response)
    }

    fn trigger_event(& self, event: &str, data: Vec<Value>, namespace: &str, sid: &str) -> tide::Result<Response>
    {
        // Is that not deadlock??
        let listeners = self.listeners.lock().unwrap();
        if let Some(callbacks) = listeners.get(event) {
            for callback in callbacks {
                callback(
                    data.clone(),
                    namespace.to_string(),
                    sid.to_string()
                ); // Execute each callback with the provided data
            }
        }
        return Ok(Response::builder(StatusCode::Ok).body("").build());
    }

    fn handle_event(& self, namespace: &str, data: Vec<Value>, sid: &str) -> tide::Result<Response> {
        if let Some(first_value) = data.clone().first() {
            if let Value::String(ref key) = first_value {
                
                let contains_key = {
                    let listeners = self.listeners.lock().unwrap();
                    listeners.contains_key(key)
                };

                if contains_key {
                    self.trigger_event(key,data,namespace,sid);
                }
            }
            // Check if the first value is a string and convert it
        }
        return Ok(Response::builder(StatusCode::Ok).body("Event processed").build());
    }

    pub async fn handle(& self, payload: &str, sid: &str) -> tide::Result<Response>
    {

        println!("payload: {}",payload);
        let packet = decode_packet(payload).unwrap();
        match packet {
            Packet::CONNECT { namespace, data, id } => {return self.handle_connect(namespace.as_str(),data,sid)},
            Packet::DISCONNECT { namespace, data, id } => {return self.handle_disconnect(namespace.as_str(),data,sid);},
            Packet::EVENT { namespace, data, id } => {return self.handle_event(namespace.as_str(),data,sid);},
            Packet::ACK { namespace, data, id } => {println!("boucing ACK");},
            Packet::CONNECT_ERROR { namespace, data, id } => {println!("boucing CONNECT_ERROR");},
            Packet::BINARY_EVENT { namespace, data, id } => {println!("boucing EVENT");},
            Packet::BINARY_ACK { namespace, data, id } => {println!("boucing ACK");},//handle_noop(&payload)},
            // Handle other packet types as needed
            _ => {println!("Unknown packet type: {}", payload);},
        }

        return Ok(Response::builder(StatusCode::Ok).body("Messages processed").build());
    }
}

const EIO_OPEN: &str = "0";
const EIO_CLOSE: &str = "1";
const EIO_PING: &str = "2";
const EIO_PONG: &str = "3";
const EIO_MESSAGE: &str = "4";
const EIO_UPGRADE: &str = "5";
const EIO_NOOP: &str = "6";

#[derive(Clone)]
pub struct EngineIO {
    socketio : Arc<SocketIO>,
    // listeners : Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(String) + Send>>>>>,
}
impl EngineIO {
    pub fn new() -> Self
    {
        Self {
            socketio : Arc::new(SocketIO::new()),
            // listeners : Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Function to add a listener (for testing purposes)
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(Vec<Value>,String, String) + Send + 'static,
    {
        self.socketio.on(event,callback);
    }

    pub fn emit(&self, event: &str, data: Vec<Value>, namespace: String, sid: String)
    {
        self.socketio.emit(event, data, namespace, sid);
    }

    pub async fn handle_engine_io(&self, req: tide::Request<()>) -> tide::Result<Response>
    {
        self.handle_long_polling(req).await
    }

    fn handle_open(&self, payload: &str) -> tide::Result<Response>
    {
        println!("Open");
        return Ok(Response::builder(StatusCode::Ok).body("Messages processed").build());
    }

    async fn handle_message(&self, payload: &str, sid: &str) -> tide::Result<Response>
    {
        println!("message: {}", payload);
        return self.socketio.handle(payload,sid).await;
    }

    fn ping(&self) ->  tide::Result<Response>
    {
        let mut response = Response::new(200);
        let response_body = format!("{}", EIO_PING);
        response.set_body(response_body);
        response.insert_header("Content-Type", "application/json");
        Ok(response)
    }

    fn pong(&self) ->  tide::Result<Response>
    {
        let mut response = Response::new(200);
        let response_body = format!("{}", EIO_PONG);
        response.set_body(response_body);
        response.insert_header("Content-Type", "application/json");
        Ok(response)
    }

    fn set_connect(& self, sid: &str){
        self.socketio.set_connect(sid);
    }

    fn set_disconnect(& self, sid: &str){
        self.socketio.set_disconnect(sid);
    }

    pub fn is_connected(& self, sid: &str) -> bool {
        self.socketio.is_connected(sid) && self.socketio.is_registered_namespace(sid)
    }

    async fn handle_packet(&self, packet: &str, sid: &str) -> tide::Result<Response>{

        let packet_type = &packet[0..1];
        let payload = &packet[1..];
        // println!("packet: {} {}",packet_type, payload);
        match packet_type {
            EIO_OPEN => {self.handle_open(&payload);},
            EIO_CLOSE => {println!("boucing CLOSE");},
            EIO_PING => {return self.pong();},
            EIO_PONG => {return self.ping();},
            EIO_MESSAGE => {return self.handle_message(&payload,sid).await;},
            EIO_UPGRADE => {println!("boucing UPGRADE");},
            EIO_NOOP => {println!("boucing NOOP");},//handle_noop(&payload)},
            // Handle other packet types as needed
            _ => println!("Unknown packet type: {}", packet_type),
        }

        // Return the JSON response
        let mut response = Response::new(200);
        response.insert_header("Content-Type", "application/json");
        return Ok(response);
    }

    async fn handle_long_polling(&self, mut req: tide::Request<()>) ->  tide::Result<Response>
    {
        let body: String = req.body_string().await?;
        let q: Query = req.query().unwrap();
        let packets: Vec<&str> = body.split('\x1e').collect();
        let sid_q = if q.sid.is_empty() { &q.t.clone() } else { &q.sid.clone() };
        
        // println!("eio:{} transport:{} t:{} sid:{}",q.EIO, q.transport, q.t, q.sid);
        for packet in packets {
            // println!("handle packet {}",packet);
            if !packet.is_empty() {
                return self.handle_packet(&packet, &sid_q).await;
            }
        }

        // Create the response JSON
        let mut response = Response::new(200);
        let max_duration_ms = 20000;
        if q.sid.is_empty() {

            
            let response_json = json!({
                "sid": sid_q,
                "upgrades": [""],
                "pingInterval": 25000,
                "pingTimeout": max_duration_ms,
                "maxPayload": 1000000
            });
    
            // Return the JSON response
            let response_body = format!("0{}", serde_json::to_string(&response_json)?);
            response.set_body(response_body);
            response.insert_header("Content-Type", "text/plain; charset=UTF-8");
        }
        else{
            // long polling
            let start_time = Instant::now();
            if self.is_connected(sid_q.as_str()) {

                while start_time.elapsed() < Duration::from_millis(max_duration_ms)
                {
                    if let Some(item) = self.socketio.get_payload(q.sid.as_str())
                    {
                        // TODO: process data
                        response.set_body(item.data);
                        response.insert_header("Content-Type", "text/plain; charset=UTF-8");
                        return Ok(response);
                    }
                }
                // else send just ping if no data to send
                return self.ping();
            }
            else
            {
                self.set_connect(sid_q);
                return self.handle_message("0",sid_q).await;
            }
        }

        Ok(response)
    }
}


/// Unit tests:
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn decode_IO_CONNECT() {
        let encoded_connect = "0/admin,{\"sid\":\"oSO0OpakMV_3jnilAAAA\"}";
        let result = decode_packet(encoded_connect);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::CONNECT { namespace, data, id } => {
                println!("{}",data);
                assert_eq!(namespace, "/admin"); // Adjust based on your implementation
                assert_eq!(data, json!({"sid":"oSO0OpakMV_3jnilAAAA"})); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_IO_EVENT() {
        let encoded_event = "2/admin,[\"bar\"]";
        let result = decode_packet(encoded_event);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::EVENT { namespace, data, id } => {
                assert_eq!(namespace, "/admin"); // Adjust based on your implementation
                assert_eq!(data, vec![Value::String("bar".to_string())]); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant"),
        }
    }
    #[test]
    fn decode_IO_EVENT_LONG() {
        let encoded_event = "2/admin,[\"bar\",\"bar1\",\"bar2\"]";
        let result = decode_packet(encoded_event);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::EVENT { namespace, data, id } => {
                assert_eq!(namespace, "/admin"); // Adjust based on your implementation
                assert_eq!(data, vec![Value::String("bar".to_string()),
                                      Value::String("bar1".to_string()),
                                      Value::String("bar2".to_string())]); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant"),
        }
    }
    #[test]
    fn decode_IO_EVENT_REAL() {
    let encoded_event = "2[\"bind\",{\"user_id\":\"1\"}]";
    let result = decode_packet(encoded_event);
    assert!(result.is_ok());
    let packet = result.unwrap();
    match packet {
        Packet::EVENT { namespace, data, id } => {
            assert_eq!(namespace, "/"); // Adjust based on your implementation
            assert_eq!(data, vec![Value::String("bind".to_string()),
                                  json!({"user_id": "1"})]); // Check if data matches the expected empty JSON object
        },
        _ => panic!("Expected ACK packet but got a different variant"),
    }
}

    #[test]
    fn decode_IO_CONNECT_ERROR() {
        let encoded_connect_error = "4{\"message\":\"Not authorized\"}";
        let result = decode_packet(encoded_connect_error);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::CONNECT_ERROR { namespace, data, id } => {
                println!("{}",data);
                assert_eq!(namespace, "/"); // Adjust based on your implementation
                assert_eq!(data, json!({"message":"Not authorized"})); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_IO_CONNECT_DEFAULT() {
        let encoded_connect_error = "0{\"message\":\"Not authorized\"}";
        let result = decode_packet(encoded_connect_error);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::CONNECT { namespace, data, id } => {
                println!("{}",data);
                assert_eq!(namespace, "/"); // Adjust based on your implementation
                assert_eq!(data, json!({"message":"Not authorized"})); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_IO_ACK_PAYLOAD() {
        let encoded_ack = "3{\"message\":\"Not authorized\"}";
        let result = decode_packet(encoded_ack);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::ACK { namespace, data, id } => {
                println!("{}",data);
                assert_eq!(namespace, "/"); // Adjust based on your implementation
                assert_eq!(data, json!({"message":"Not authorized"})); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_IO_ACK() {
        let encoded_ack = "3";
        let result = decode_packet(encoded_ack);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::ACK { namespace, data, id } => {
                println!("{}",data);
                assert_eq!(namespace, "/"); // Adjust based on your implementation
                assert_eq!(data, json!({})); // Check if data matches the expected empty JSON object
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_BINARY_EVENT_1() {
        let encoded_ack = "51-[\"baz\",{\"_placeholder\":true,\"num\":0}]
                            + <Buffer <01 02 03 04>>";
        let result = decode_packet(encoded_ack);
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::BINARY_EVENT { namespace, data, id } => {
                assert_eq!(namespace, "/"); // Adjust based on your implementation
                assert_eq!(data, vec![
                    String::from("baz"),
                    String::from("AQIDBA==")
                ]); // Check if vector data is ok
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_BINARY_EVENT_2() {
        let encoded_ack = "52-/admin,[\"baz\",{\"_placeholder\":true,\"num\":0},{\"_placeholder\":true,\"num\":1}]
                        + <Buffer <01 02>>
                        + <Buffer <03 04>>";
        let result = decode_packet(encoded_ack);
        println!("{}",result.is_ok());
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::BINARY_EVENT { namespace, data, id } => {
                println!("{:?}",data);
                assert_eq!(namespace, "/admin"); // Adjust based on your implementation
                assert_eq!(data, vec![
                    String::from("baz"),
                    String::from("AQI="),
                    String::from("AwQ=")
                ]); // Check if vector data is ok
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn decode_BINARY_EVENT_ID_2() {
        let encoded_ack = "52-/admin,12[\"baz\",{\"_placeholder\":true,\"num\":0},{\"_placeholder\":true,\"num\":1}]
                        + <Buffer <01 02>>
                        + <Buffer <03 04>>";
        let result = decode_packet(encoded_ack);
        println!("{}",result.is_ok());
        assert!(result.is_ok());
        let packet = result.unwrap();
        match packet {
            Packet::BINARY_EVENT { namespace, data, id } => {
                assert_eq!(namespace, "/admin"); // Adjust based on your implementation
                assert_eq!(data, vec![
                    String::from("baz"),
                    String::from("AQI="),
                    String::from("AwQ=")
                ]); // Check if vector data is ok
                // TODO: ID is not stored yet
                assert_eq!(id,String::from("12"));
            },
            _ => panic!("Expected ACK packet but got a different variant: {:?}", packet),
        }
    }
    #[test]
    fn encode_IO_EVENT() {
        let payload = vec![Value::String("bar".to_string())];
        let packet = Packet::EVENT{
            id : String::from(""),
            namespace : String::from("/admin"),
            data : payload.clone()
        };
        let encoded_result = encode_packet(packet).unwrap();

        let encoded_event = String::from("42/admin,[\"bar\"]");
        assert_eq!(encoded_result,encoded_event);
    }
    #[test]
    fn encode_IO_EVENT_LONG() {
        let payload = vec![Value::String("bar".to_string()),
        Value::String("bar1".to_string()),
        Value::String("bar2".to_string())];
        
        let packet = Packet::EVENT{
            id : String::from(""),
            namespace : String::from("/admin"),
            data : payload.clone()
        };
        let encoded_result = encode_packet(packet).unwrap();

        let encoded_event = String::from("42/admin,[\"bar\",\"bar1\",\"bar2\"]");
        assert_eq!(encoded_result,encoded_event);
    }
    
}