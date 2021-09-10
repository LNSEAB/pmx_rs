fn main() {
    let file = std::fs::File::open("resource/Alicia/Alicia_solid.pmx").unwrap();
    let reader = std::io::BufReader::new(file);
    let pmx = pmx_rs::read(reader).unwrap();
    println!("{}", pmx.model_info.name);
    println!("vertex: {}", pmx.vertices.len());
    println!("face: {}", pmx.faces.len());
    println!("material: {}", pmx.materials.len());
    for mat in pmx.materials.iter() {
        println!("{}", mat.name);
    }
    println!("bone: {}", pmx.bones.len());
    for bone in pmx.bones.iter() {
        println!("{}", bone.name);
    }
    println!("rigid: {}", pmx.rigids.len());
    for rigid in pmx.rigids.iter() {
        println!("{}", rigid.name);
    }
    println!("joint: {}", pmx.joints.len());
    for joint in pmx.joints.iter() {
        println!("{}", joint.name);
    }
}
