pub struct Config {
    pub sea_level: f32,
    pub mountain_scale: f32,
    pub snow_temp: f32,
    pub tropical_temp: f32,
    pub desert_temp: f32,
}

pub const CONFIG: Config = Config {
    sea_level: 140.0,
    mountain_scale: 1000.0,
    snow_temp: -0.4,
    tropical_temp: 0.25,
    desert_temp: 0.45,
};
