pub mod from_client {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct Greeting {
        pub name: String,
    }
}

pub mod from_server {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct GreetingResponse {
        pub player_id: usize
    }
}

pub mod both {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct PlayerStatus {
        pub player_id: usize,
        pub x: f64,
        pub y: f64,
        pub angle: f64,
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub holster: bool,
    }
}