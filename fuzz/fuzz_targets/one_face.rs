#![no_main]

#[path = "../src/lib.rs"]
mod geometry;

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;

use crate::geometry::{Face, Geometry, Vertex};

#[derive(Debug)]
struct OneFace(Geometry);

impl Arbitrary<'_> for OneFace {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, arbitrary::Error> {
        let vertices = Vec::<Vertex>::arbitrary(u)?;
        let faces = vec![Face {
            vertex_indices: (0..vertices.len()).collect(),
        }];
        let mut value = Geometry { vertices, faces };

        value.validate()?;

        Ok(Self(value))
    }
}

fuzz_target!(|value: OneFace| {
    let OneFace(value) = value;

    let reference = {
        let mut value = value.clone();
        mikktspace_sys::gen_tang_space_default(&mut value);
        value
    };

    let value = {
        let mut value = value;
        bevy_mikktspace::generate_tangents(&mut value);
        value
    };

    if reference != value {
        let r = reference.vertices.iter().flat_map(|v| v.tangent);
        let v = value.vertices.iter().flat_map(|v| v.tangent);

        r.zip(v)
            .for_each(|(r, v)| assert_eq!(r.to_ne_bytes(), v.to_ne_bytes(), "{} != {}", r, v));

        assert_eq!(
            reference, value,
            "tangents equal, but something else was changed improperly!"
        );
    }
});
