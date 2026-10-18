mod dbclient;
pub mod server;

pub mod proto {
    pub mod connections {
        include!(concat!(env!("OUT_DIR"), "/dbclient.connections.rs"));
    }
    pub mod objects {
        include!(concat!(env!("OUT_DIR"), "/dbclient.objects.rs"));
    }
    pub mod queries {
        include!(concat!(env!("OUT_DIR"), "/dbclient.queries.rs"));
    }
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/dbclient.common.rs"));
    }
    pub mod postgresql {
        include!(concat!(env!("OUT_DIR"), "/dbclient.postgresql.rs"));
    }
    pub mod redis {
        include!(concat!(env!("OUT_DIR"), "/dbclient.redis.rs"));
    }
}
