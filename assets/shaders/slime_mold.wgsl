@group(0) @binding(0)
var trailMap: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0)
var<storage, read_write> agents: array<vec3<f32>>;

@group(2) @binding(0)
var<uniform> settings: SettingsUniform;


struct SettingsUniform {
    dimX: i32,
    dimY: i32,
    deltaTime: f32,
    time: f32,

    moveSpeed: f32,
    turnSpeed: f32,

    trailWeight: f32,
    decayRate: f32,
    diffuseRate: f32,

    sensorAngleSpacing: f32,
    sensorOffsetDst: f32,
    sensorSize: i32,
// #ifdef SIXTEEN_BYTE_ALIGNMENT
//     _padding: vec3<f32>,
// #endif
}


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

fn scale01(value: u32) -> f32 {
    return f32(value) / 4294967295.0;
}

@compute @workgroup_size(16, 1, 1)
fn initAgents(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let randomRadius = random(randomState) * 70.0;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * randomRadius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle);
}

@compute @workgroup_size(16, 1, 1)
fn updateAgents(@builtin(global_invocation_id) id: vec3<u32>) {

    let agent = agents[id.x];
    let pos = vec2<f32>(agent.x, agent.y);
    let angle = agent.z;

    var rng = hash(u32(i32(pos.y) * settings.dimX + i32(pos.x)) + hash(id.x + u32(settings.time) * 100000u));

    let direction = vec2<f32>(cos(angle), sin(angle));
    var newPos = pos + direction * settings.deltaTime * settings.moveSpeed;
    var newAngle = angle;

    if (newPos.x < 0.0 || newPos.x >= f32(settings.dimX) || newPos.y < 0.0 || newPos.y >= f32(settings.dimY)) {
        rng = hash(rng);
        let randAngle = scale01(rng) * TAU;

        newPos.x = min(f32(settings.dimX - 1), max(0.0, newPos.x));
        newPos.y = min(f32(settings.dimY - 1), max(0.0, newPos.y));
        newAngle = randAngle;
    } else {
        let location = vec2<i32>(i32(newPos.x), i32(newPos.y));
        let oldTrail = textureLoad(trailMap, location);

        storageBarrier();
        textureStore(trailMap, location, min(vec4<f32>(1.0), oldTrail + settings.trailWeight * settings.deltaTime));
    }
    storageBarrier();

    agents[id.x] = vec3<f32>(newPos.x, newPos.y, newAngle);
}


@compute @workgroup_size(8, 8, 1)
fn updateTrailmap(@builtin(global_invocation_id) id: vec3<u32>) {

    // let agent = agents[id.x];
    // let pos = vec2<f32>(agent.x, agent.y);
    // let angle = agent.z;

    // let direction = vec2<f32>(cos(angle), sin(angle));
    // let newPos = pos + direction;

    let location = vec2<i32>(i32(id.x), i32(id.y));

    // storageBarrier();

    // textureStore(trailMap, location, vec4<f32>(1.0, 0.0, 0.0, 1.0));
}