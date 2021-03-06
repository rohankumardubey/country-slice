use bevy_ecs::prelude::*;

use crate::{
    asset_libraries::{
        mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary,
        vao_library::AssetVAOLibrary, Handle,
    },
    render::{mesh::Mesh, shader::ShaderProgram, vao::VAO},
};

pub fn build_missing_vaos(
    query: Query<(Entity, &Handle<Mesh>, &Handle<ShaderProgram>), Without<Handle<VAO>>>,
    assets_mesh: Res<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
    mut assets_vao: ResMut<AssetVAOLibrary>,
    mut commands: Commands,
) {
    for (ent, mesh_handle, shader_handle) in query.iter() {
        // Check if mesh is not empty
        if assets_mesh
            .get(*mesh_handle)
            .map(|mesh| mesh.indices.is_empty())
            .unwrap_or(false)
        {
            log::debug!("Mesh is empty. Skip building VAO");
            continue;
        }

        match assets_vao.add(&assets_mesh, &assets_shader, *mesh_handle, *shader_handle) {
            Err(error) => {
                log::warn!("Couldn't build VAO: {}", error)
            }
            Ok(vao_handle) => {
                log::debug!("Built VAO {:?} for {:?}", vao_handle, mesh_handle);
                // add component
                commands.entity(ent).insert(vao_handle);
            }
        }
    }
}

pub fn rebuild_vaos(
    assets_shader: Res<AssetShaderLibrary>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
    mut assets_vao: ResMut<AssetVAOLibrary>,
) {
    if assets_mesh.marked_as_dirty.is_empty() {
        return;
    }

    let mut still_need_rebuilding = Vec::new();
    for dirty_handle in &assets_mesh.marked_as_dirty {
        if assets_vao.has_vao(*dirty_handle) {
            match assets_vao.rebuild_vao(&assets_mesh, &assets_shader, *dirty_handle) {
                Err(error) => {
                    still_need_rebuilding.push(*dirty_handle);
                    log::warn!("Couldn't rebuild VAO: {}", error)
                }
                Ok(_) => (), //println!("Rebuilt VAOs for {:?}", dirty_handle),
            }
        } else {
            log::warn!("WARNING: Tried rebuilding a mesh that has no VAOs!");
        }
    }
    assets_mesh.marked_as_dirty = still_need_rebuilding;
}
