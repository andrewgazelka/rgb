mod simulation;

pub use simulation::{
    check_edge_activity, compute_next_generation, count_neighbors, expand_world, get_cell,
    normalize_coords, register_life_systems,
};
