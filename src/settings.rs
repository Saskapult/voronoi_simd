use glam::Vec2;


pub struct VoronoiSettings {
	seed: u32,
	frequency: Vec2, 
	offset: Vec2,
}


trait VoronoiDistance {
	fn distance(point: Vec2, centre: Vec2) -> f32;
}

pub struct VoronoiDistanceEuclidean;
impl VoronoiDistance for VoronoiDistanceEuclidean {
	fn distance(point: Vec2, centre: Vec2) -> f32 {
		(point - centre).length()
	}
}
