# Slime Mold Simulation

## Description

A slime mold simulation writting in Rust with Wgpu. 

### Features

All of the calculation runs in parallel on the GPU using compute shaders, so it runs really fast. 
Initial state of agents can be changed in `/src/slime_mold/mod.rs`. Note that it must be the name of an initial function present in the shader.
Valid functions are: 
- `initAgentsInwardCircle`
- `initAgentsOutwardCircle`
- `initAgentsInwardRing`
- `initAgentsOutwardRing`
- `initAgentsPoint`

### Screenshots

![Alt text](/screenshots/inward_ring.png?raw=true "Inward Ring")
![Alt text](/screenshots/outward_circle.png?raw=true "Outward Circle")