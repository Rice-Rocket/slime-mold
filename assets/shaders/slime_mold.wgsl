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

    colorA: vec4<f32>,
    colorB: vec4<f32>,
// #ifdef SIXTEEN_BYTE_ALIGNMENT
//     _padding: vec3<f32>,
// #endif
}


const TAU: f32 = 6.283185;
const PI: f32 = 3.1415927;

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
fn initAgentsInwardCircle(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let randomRadius = random(randomState) * f32(settings.dimY) * 0.4;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * randomRadius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle - PI);
}
@compute @workgroup_size(16, 1, 1)
fn initAgentsOutwardCircle(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let randomRadius = random(randomState) * f32(settings.dimY) * 0.3;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * randomRadius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle);
}
@compute @workgroup_size(16, 1, 1)
fn initAgentsInwardRing(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let radius = f32(settings.dimY) * 0.4;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * radius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle - PI);
}
@compute @workgroup_size(16, 1, 1)
fn initAgentsOutwardRing(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let radius = f32(settings.dimY) * 0.4;
    let randomAngle = random(randomState * 2u) * TAU;

    let randomPosition = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0) + vec2<f32>(cos(randomAngle), sin(randomAngle)) * radius;

    storageBarrier();

    agents[agentIdx] = vec3<f32>(randomPosition.x, randomPosition.y, randomAngle);
}
@compute @workgroup_size(16, 1, 1)
fn initAgentsPoint(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let agentIdx: u32 = id.x;
    let randomState: u32 = num_workgroups.x * id.x;

    let randomAngle = random(randomState * 2u) * TAU;
    let position = vec2<f32>(f32(settings.dimX) / 2.0, f32(settings.dimY) / 2.0);

    storageBarrier();

    agents[agentIdx] = vec3<f32>(position.x, position.y, randomAngle);
}


fn sense(agent: vec3<f32>, sensorAngleOffset: f32) -> f32 {
    let sensorAngle = agent.z + sensorAngleOffset;
    let sensorDir = vec2<f32>(cos(sensorAngle), sin(sensorAngle));

    let sensorPos = agent.xy + sensorDir * settings.sensorOffsetDst;
    let sensorCenterX = i32(sensorPos.x);
    let sensorCenterY = i32(sensorPos.y);

    var sum = 0.0;
    for (var offsetX = -settings.sensorSize; offsetX <= settings.sensorSize; offsetX++) {
        for (var offsetY = -settings.sensorSize; offsetY <= settings.sensorSize; offsetY++) {
            let sampleX = min(settings.dimX - 1, max(0, sensorCenterX + offsetX));
            let sampleY = min(settings.dimY - 1, max(0, sensorCenterY + offsetY));
            sum += textureLoad(trailMap, vec2<i32>(sampleX, sampleY)).w;
        }
    }
    return sum;
}

@compute @workgroup_size(16, 1, 1)
fn updateAgents(@builtin(global_invocation_id) id: vec3<u32>) {

    let agent = agents[id.x];
    let pos = vec2<f32>(agent.x, agent.y);
    let angle = agent.z;

    var rng = hash(u32(i32(pos.y) * settings.dimX + i32(pos.x)) + hash(id.x + u32(settings.time) * 100000u));

    let sensorAngleRad = settings.sensorAngleSpacing * (PI / 180.0);
    let weightForward = sense(agent, 0.0);
    let weightLeft = sense(agent, sensorAngleRad);
    let weightRight = sense(agent, -sensorAngleRad);

    let steerStrength = scale01(rng);
    let turnSpeed = settings.turnSpeed * TAU;

    var newAngle = angle;

    if weightForward > weightLeft && weightForward > weightRight {
        newAngle += 0.0;
    }
    else if weightForward < weightLeft && weightForward < weightRight {
        newAngle += (steerStrength - 0.5) * 2.0 * turnSpeed * settings.deltaTime;
    }
    else if weightRight > weightLeft {
        newAngle -= steerStrength * turnSpeed * settings.deltaTime;
    }
    else if weightRight < weightLeft {
        newAngle += steerStrength * turnSpeed * settings.deltaTime;
    }

    let direction = vec2<f32>(cos(angle), sin(angle));
    var newPos = pos + direction * settings.deltaTime * settings.moveSpeed;

    if (newPos.x < 0.0 || i32(newPos.x) >= settings.dimX || newPos.y < 0.0 || i32(newPos.y) >= settings.dimY) {
        rng = hash(rng);
        let randAngle = scale01(rng) * TAU;

        newPos.x = min(f32(settings.dimX - 1), max(0.0, newPos.x));
        newPos.y = min(f32(settings.dimY - 1), max(0.0, newPos.y));
        newAngle = randAngle;
    } else {
        let location = vec2<i32>(newPos);
        let oldTrail = textureLoad(trailMap, location);

        storageBarrier();
        textureStore(trailMap, location, min(settings.colorA, oldTrail + settings.trailWeight * settings.deltaTime));
    }
    storageBarrier();
    agents[id.x] = vec3<f32>(newPos.x, newPos.y, newAngle);
}


@compute @workgroup_size(8, 8, 1)
fn updateTrailmap(@builtin(global_invocation_id) id: vec3<u32>) {
    let location = vec2<i32>(i32(id.x), i32(id.y));

    // if (location.x < 0 || location.x >= settings.dimX || location.y < 0 || location.y >= settings.dimY) {
    //     return;
    // }

    var sum = 0.0;
    let oldColor = textureLoad(trailMap, location).w;

    for (var offsetX = -1; offsetX <= 1; offsetX++) {
        for (var offsetY = -1; offsetY <= 1; offsetY++) {
            let sampleX = min(settings.dimX - 1, max(0, location.x + offsetX));
            let sampleY = min(settings.dimY - 1, max(0, location.y + offsetY));
            sum += textureLoad(trailMap, vec2<i32>(sampleX, sampleY)).w;
        }
    }

    let blurred = sum / 9.0;
    let diffuseWeight = saturate(settings.diffuseRate * settings.deltaTime);
    let finalBlurred = oldColor * (1.0 - diffuseWeight) + blurred * diffuseWeight;
    let finalValue = finalBlurred - settings.decayRate * settings.deltaTime;
    let finalCol = settings.colorB + (settings.colorA - settings.colorB) * max(0.0, min(1.0, finalValue));

    storageBarrier();
    textureStore(trailMap, location, vec4<f32>(finalCol.xyz, max(0.0, finalValue)));
}