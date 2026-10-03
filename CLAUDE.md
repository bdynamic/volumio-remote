This file provides guidance for AI Assistants

## Project short description
volumio-remote: lightweight Linux desktop remote for a Volumio instance on the local network (Rust + Slint). Now-playing window with cover art, tray icon, MPRIS media keys and a virtual audio sink for the volume knob.

## General
- Be brief, techn. precise, direct, honest, no flattery
- If documentatation creation is needed create them in `devdoc/`, prefer small distinc file per topic over single large ones. Prefer tables and diagrams
- You can only add links in section below, not change this document otherwise
- git user name: Auto Coder, auto.coder@dy-mail.de
- you are running in a Docker container
- Required tools (e.g. C compiler/linker, build libs) shall be installed in the container
- Req/arch docs = living. Keep aligned w/ reality. Document every relevant change.

## Development Approach
- This is a POC/MVP, NOT an enterprise project
- Start with the simplest solution that works
- Avoid frameworks unless absolutely necessary
- Prefer single-file implementations when feasible
- Hardcode reasonable defaults instead of complex config systems
- Don't add abstractions until genuinely needed
- Skip complex error handling for unlikely edge cases
- Don't optimize prematurely
- If in a git repo commit after each major step
- Never commit file which contain personal data or crypt. artifacts. Put them to gitignore, create samples and commit
- Update the documentation files after finishing and verifing a feature

## Links (`devdoc/`)
- to be filled by Coding assistant
- [VolumioX research](devdoc/volumiox-research.md)
- [Requirements](devdoc/requirements.md)
- [Tray reference](devdoc/tray-reference.md)
- [Build and install](devdoc/build-install.md)
- [Architecture](devdoc/architecture.md)
- [Volume knob](devdoc/volume-knob.md)
