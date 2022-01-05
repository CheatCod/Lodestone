![workflow](https://github.com/CheatCod/Lodestone/actions/workflows/rust.yml/badge.svg)
![workflow](https://github.com/CheatCod/Lodestone/actions/workflows/node.js.yml/badge.svg)

# Lodestone

## What is Lodestone?
Lodestone is a wrapper tool that aims to provide an easy and consistent server hosting experience for Minecraft.

**Lodestone is still in the very early stage of development, as such you should use it ONLY if you know what you are doing**

---

## Setup
### Ubuntu:

Run the ```dev_setup.sh``` script, this will install all the dependencies.

To run the front end, `cd` into `frontend` and run `npm i && npm start`

To run the back end, `cd` into `backend`, make the following directories with `mkdir InstanceTest/db`, and run `mongod --dbpath InstanceTest/db` . Finally, on a seperate terminal, run `cargo run`

### Windows:
Windows support is planned.

---

[Trello](https://trello.com/b/sCaSEPyU/lodestone)
[Figma](https://www.figma.com/file/gM7KUynANg4JkGF3QBsYJ9/Lodestone?node-id=166%3A1621)
