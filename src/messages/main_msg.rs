// main_msg
use crate::messages::{
    requests::request_struct::Request,
    events::event::Event,
    commands::command::Command,
};



pub enum MainMsg{
    Event(Event),
    Request(Request),
    Command(Command)
}






