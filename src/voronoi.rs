use std::hash::Hasher;
use std::ops::{BitOr, BitXor, Shr};
use std::simd::num::{SimdFloat, SimdUint};
use std::simd::{self, f32x8, u32x8, StdFloat};
use fnv::FnvHasher;
use glam::Vec2;


// Returns the distance to the closest cell and the cell's position
pub fn voronoi_basic(
	seed: u32,
	frequency: f32, 
	x_offset: f32, 
	y_offset: f32,
) -> (f32, Vec2) {
	// Scale pos by frequency 
	let scaled_pos = Vec2::new(x_offset, y_offset) * frequency;

	// Find the cell position
	let cell_pos = scaled_pos.floor();
	// Find the cell point [0, 1]
	let pos_in_cell = scaled_pos.fract();

	let prediv_cpos = cell_pos / frequency;
	let prediv_offset = 1.0 / frequency;

	let mut mind = 42.0;
	for [x, xo] in [
		[-1.0, -prediv_offset], 
		[0.0, 0.0], 
		[1.0, prediv_offset],
	] {
		for [y, yo] in [
			[-1.0, -prediv_offset], 
			[0.0, 0.0], 
			[1.0, prediv_offset],
		] {
			// Cell-sapce offset
			let offset = Vec2::new(xo, yo);

			let neighbour_cell_pos = prediv_cpos + offset;

			let point = Vec2::new(x, y) + son_of_pos_hash(neighbour_cell_pos, FnvHasher::with_key(seed as u64));

			// Distance calculation, use funky stuff 
			let d = (pos_in_cell - point).length();

			if d < mind {
				mind = d;
			}
		}
	}

	mind
}

// // Profiling has revealed this to be around twice as slow as without SIMD
// pub fn voronoi_basic_simd(
// 	seed: u32,
// 	frequency: f32, 
// 	x_offset: f32, 
// 	y_offset: f32,
// ) -> f32 {
// 	// Scale pos by frequency 
// 	let scaled_pos = Vec2::new(x_offset, y_offset) * frequency;

// 	// Find the cell position
// 	let cell_pos = scaled_pos.floor();
// 	// Find the cell point [0, 1]
// 	let pos_in_cell = scaled_pos.fract();

// 	let prediv_cpos = cell_pos / frequency;
// 	let prediv_offset = 1.0 / frequency;

// 	// Base cell calculation
// 	let p0 = pos_hash_2d_2d(prediv_cpos);
// 	let d0 = (pos_in_cell - p0).length();

// 	// No loops, just precomputed order 
// 	let offsets_x = f32x8::from_array([-prediv_offset, -prediv_offset, -prediv_offset, 0.0, 0.0, prediv_offset, prediv_offset, prediv_offset]);
// 	let offsets_y = f32x8::from_array([-prediv_offset, 0.0, prediv_offset, -prediv_offset, prediv_offset, -prediv_offset, 0.0, prediv_offset]);

// 	let neighbour_cell_pos_x = f32x8::splat(prediv_cpos.x) + offsets_x;
// 	let neighbour_cell_pos_y = f32x8::splat(prediv_cpos.y) + offsets_y;

// 	// Offsets of the neighbouring cells 
// 	let ox = f32x8::from_array([-1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
// 	let oy = f32x8::from_array([-1.0, 0.0, 1.0, -1.0, 1.0, -1.0, 0.0, 1.0]);
// 	// Point offsets of the neighbouring cells
// 	let [px, py] = simd_fnv1a(neighbour_cell_pos_x, neighbour_cell_pos_y);
// 	// Points of the neighbouring cells relative to the centre cell
// 	let point_x = ox + px;
// 	let point_y = oy + py;

// 	// Distance to the points of the neighbouring cell
// 	let d_x = f32x8::splat(pos_in_cell.x) - point_x;
// 	let d_y = f32x8::splat(pos_in_cell.y) - point_y;
// 	let d = (d_x * d_x + d_y * d_y).sqrt();

// 	// Return min of neighbour cells and base cell
// 	let min_d = d.reduce_min();
// 	f32::min(min_d, d0)
// }

// // https://www.ronja-tutorials.com/post/024-white-noise/
// // I modified the code a little becuase the pattern resulted in broken, 
// // grid-like noise. 
// #[inline]
// fn pos_hash_2d_2d(value: Vec2) -> Vec2 {
// 	#[inline]
// 	fn pos_hash_2d_1d(value: f32, dot_dir: Vec2) -> f32 {
// 		let small_value = value.sin();
// 		let random = dot_dir.dot(Vec2::splat(small_value));
// 		(random.sin() * 143758.5453).abs().fract()
// 	}

// 	Vec2::new(
// 		pos_hash_2d_1d(value.x + value.y.sin(), Vec2::new(12.989, 78.233)),
// 		pos_hash_2d_1d(value.x.sin() + value.y, Vec2::new(39.346, 11.135)),
// 	)
// }

// #[inline]
// fn pos_hash_2d_2d_simd_8(value_x: f32x8, value_y: f32x8) -> [f32x8; 2] {
// 	#[inline]
// 	fn pos_hash_2d_1d_simd_8(value: f32x8, dot_dir: Vec2) -> f32x8 {
// 		let sins = value.sin();

// 		let dot_x = f32x8::splat(dot_dir.x);
// 		let dot_y = f32x8::splat(dot_dir.y);

// 		let random = dot_x * sins + dot_y * sins;
// 		(random.sin() * f32x8::splat(143758.5453)).abs().fract()
// 	}

// 	[
// 		pos_hash_2d_1d_simd_8(value_x + value_y.sin(), Vec2::new(12.989, 78.233)),
// 		pos_hash_2d_1d_simd_8(value_x.sin() + value_y, Vec2::new(39.346, 11.135)),
// 	]
// }

// // A function made with half-remembered knowledge from CSC 349A 
// // If is far faster than pos_hash_2d_2d 
// #[inline]
// fn son_of_pos_hash<H: Hasher>(pos: Vec2, mut h: H) -> Vec2 {
// 	h.write_u32(pos.x.to_bits());
// 	h.write_u32(pos.y.to_bits());
// 	let v = h.finish();

// 	let v0 = (v >> 0) & 0xFFFFFFFF;
// 	let v1 = (v >> 32) ^ v0;

// 	Vec2::new(
// 		int_to_float(v0 as u32),
// 		int_to_float(v1 as u32),
// 	)
// }

// // Takes the most significant 23 bits from a u32 to make an f32 in [0, 1)
// // https://blog.bithole.dev/blogposts/random-float/
// #[inline]
// fn int_to_float(i: u32) -> f32 {
// 	f32::from_bits(i >> 9 | 0x3f800000) - 1.0
// }

// // http://www.isthe.com/chongo/tech/comp/fnv/index.html
// #[inline]
// fn simd_fnv1a(value_x: f32x8, value_y: f32x8) -> [f32x8; 2] {
// 	let mut hash = u32x8::splat(2166136261);
// 	for v in [value_x.to_bits(), value_y.to_bits()] {
// 		hash = hash.bitxor(v);
// 		hash = hash * u32x8::splat(0x93);
// 	}

// 	let v0 = hash;
// 	let v1 = hash.reverse_bits();
	
// 	[
// 		f32x8::from_bits(v0.shr(9).bitor(u32x8::splat(0x3f800000))) - f32x8::splat(1.0),
// 		f32x8::from_bits(v1.shr(9).bitor(u32x8::splat(0x3f800000))) - f32x8::splat(1.0),
// 	]
// }

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use test::Bencher;

// 	// Tests that our hashing is [0, 1]
// 	// Is not deterministic 
// 	#[test]
// 	fn test_pos_hash_2d_2d_bounds() {
// 		for _ in 0..8192 {
// 			let value = rand::random();
// 			let h = pos_hash_2d_2d(value);
// 			for v in h.to_array() {
// 				assert!(v >= 0.0);
// 				assert!(v <= 1.0);
// 			}
// 		}
// 	}

// 	#[test]
// 	fn test_pos_hash_2d_2d_simd_same() {
// 		for _ in 0..8192 {
// 			let values = rand::random::<[Vec2; 8]>();

// 			let bases = values.map(|value| pos_hash_2d_2d(value));

// 			let simds = {
// 				let xs = values.map(|value| value.x);
// 				let ys = values.map(|value| value.y);

// 				let x = f32x8::from_array(xs);
// 				let y = f32x8::from_array(ys);

// 				let [vx, vy] = pos_hash_2d_2d_simd_8(x, y);

// 				let mut res = [Vec2::ZERO; 8];
// 				for i in 0..res.len() {
// 					res[i].x = vx[i];
// 					res[i].y = vy[i];
// 				}
// 				res
// 			};

// 			for i in 0..bases.len() {
// 				for j in 0..2 {
// 					let a = bases[i][j];
// 					let b = simds[i][j];
// 					assert!((a - b).abs() < 0.01, "Base and SIMD voronoi differ! ({a} vs {b})");
// 				}
// 			}
// 		}
// 	}

// 	#[test]
// 	fn test_voronoi_basic_simd_same() {
// 		for _ in 0..8192 {
// 			let value = rand::random::<Vec2>() * Vec2::splat(42000.0);
// 			let base = voronoi_basic(0, 0.1, value.x, value.y);
// 			let simd = voronoi_basic_simd(0, 0.1, value.x, value.y);

// 			assert!((base - simd).abs() < 0.01, "Base and SIMD voronoi differ! ({base} vs {simd})");
// 		}
// 	}

// 	#[bench]
// 	fn bench_voronoi_basic(b: &mut Bencher) {
// 		b.iter(|| {
// 			voronoi_basic(0, 0.125, rand::random::<f32>() * 4200.0, rand::random::<f32>() * 4200.0)
// 		})
// 	}

// 	#[bench]
// 	fn bench_voronoi_basic_simd(b: &mut Bencher) {
// 		b.iter(|| {
// 			voronoi_basic_simd(0, 0.125, rand::random::<f32>() * 4200.0, rand::random::<f32>() * 4200.0)
// 		})
// 	}

// 	#[bench]
// 	fn bench_voronoi_basic_1000(b: &mut Bencher) {
// 		b.iter(|| {
// 			(0..1000).map(|seed| voronoi_basic(seed, 0.125, rand::random::<f32>() * 4200.0, rand::random::<f32>() * 4200.0)).collect::<Vec<_>>()
// 		})
// 	}

// 	#[bench]
// 	fn bench_voronoi_basic_1000_simd(b: &mut Bencher) {
// 		b.iter(|| {
// 			(0..1000).map(|seed| voronoi_basic_simd(seed, 0.125, rand::random::<f32>() * 4200.0, rand::random::<f32>() * 4200.0)).collect::<Vec<_>>()
// 		})
// 	}
// }
