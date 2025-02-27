use crate::{
    messages::{
        server::{ClientMessage, Connect, Disconnect, JoinRoom, Leave},
        session::Message,
    },
    models::{RoomId, SessionId},
};
use actix::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};

pub struct ChatServer {
    sessions: HashMap<SessionId, Recipient<Message>>,
    rooms: HashMap<RoomId, HashSet<SessionId>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn send_message(&self, room: &RoomId, message: &str, wallet_pk: Option<SessionId>) {
        let sessions = match self.rooms.get(room) {
            Some(it) => it,
            _ => return,
        };
        sessions.iter().for_each(|id| {
            if let Some(addr) = self.sessions.get(id) {
                addr.do_send(Message {
                    wallet_pk: wallet_pk.clone(),
                    msg: message.into(),
                })
            }
        });
    }

    pub fn leave_room(&mut self, wallet_pk: SessionId) {
        for sessions in self.rooms.values_mut() {
            if sessions.remove(&wallet_pk) {
                // self.send_message(&room, "Someone disconnected", Some(wallet_pk.clone()));
                break;
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let Connect { id, addr } = msg;
        self.sessions.insert(id, addr);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(
        &mut self,
        Disconnect { session }: Disconnect,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.leave_room(session.clone());
        let _ = self.sessions.remove(&session);
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, Leave { session }: Leave, _ctx: &mut Self::Context) -> Self::Result {
        self.leave_room(session);
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        let ClientMessage { session, room, msg } = msg;
        self.send_message(&room, &msg, Some(session));
    }
}

impl Handler<JoinRoom> for ChatServer {
    type Result = ();

    fn handle(
        &mut self,
        JoinRoom { session, room }: JoinRoom,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.leave_room(session.clone());

        if let Some(sessions) = self.rooms.get_mut(&room) {
            sessions.insert(session);
            self.send_message(&room, "Someone Connected", None)
        } else {
            self.sessions.get_mut(&session).unwrap().do_send(Message {
                wallet_pk: None,
                msg: "The room doesn't exist".to_owned(),
            })
        }
    }
}
