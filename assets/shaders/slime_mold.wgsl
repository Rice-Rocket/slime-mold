@group(0) @binding(0)
var trailMap: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0)
var<storage, read_write> agents: array<vec3<f32>>;

const TAU: f32 = 6.283185;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn random(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(16, 1, 1)
fn initAgents(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    // let location = vec2<i32>(i32(id.x), i32(id.y));

    // let randomNumber = random(id.y * num_workgroups.x + id.x);
    // let alive = randomNumber > 0.9;
    // let color = vec4<f32>(f32(alive));

    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let randomRadius = random(randomState) * 150.0;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(640.0 / 2.0, 360.0 / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * randomRadius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle);
    textureStore(trailMap, vec2<i32>(agents[agentIdx].xy), vec4<f32>(1.0, 0.0, 0.0, 1.0));
}

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: vec4<f32> = textureLoad(trailMap, location + vec2<i32>(offset_x, offset_y));
    return i32(value.x);
}

fn count_alive(location: vec2<i32>) -> i32 {
    return is_alive(location, -1, -1) +
           is_alive(location, -1,  0) +
           is_alive(location, -1,  1) +
           is_alive(location,  0, -1) +
           is_alive(location,  0,  1) +
           is_alive(location,  1, -1) +
           is_alive(location,  1,  0) +
           is_alive(location,  1,  1);
}

@compute @workgroup_size(16, 1, 1)
fn updateAgents(@builtin(global_invocation_id) id: vec3<u32>) {

    let agent = agents[id.x];
    let pos = vec2<f32>(agent.x, agent.y);
    let angle = agent.z;

    let direction = vec2<f32>(cos(angle), sin(angle));
    let newPos = pos + direction;

    let location = vec2<i32>(i32(pos.x), i32(pos.y));

    storageBarrier();

    agents[id.x] = vec3<f32>(newPos.x, newPos.y, angle);

    textureStore(trailMap, location, vec4<f32>(0.0, 0.0, 1.0, 1.0));
}


@compute @workgroup_size(8, 8, 1)
fn updateTrailmap(@builtin(global_invocation_id) id: vec3<u32>) {

    // let agent = agents[id.x];
    // let pos = vec2<f32>(agent.x, agent.y);
    // let angle = agent.z;

    // let direction = vec2<f32>(cos(angle), sin(angle));
    // let newPos = pos + direction;

    let location = vec2<i32>(i32(id.x), i32(id.y));

    // let n_alive = count_alive(location);

    // var alive: bool;
    // if (n_alive == 3) {
    //     alive = true;
    // } else if (n_alive == 2) {
    //     let currently_alive = is_alive(location, 0, 0);
    //     alive = bool(currently_alive);
    // } else {
    //     alive = false;
    // }
    // let color = vec4<f32>(f32(alive));

    // storageBarrier();

    // textureStore(trailMap, location, vec4<f32>(1.0, 0.0, 0.0, 1.0));
}