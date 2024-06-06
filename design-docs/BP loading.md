## Current system
A request is made to aysnc read the file / fetch it from webserver
aka we don't know when it will be ready so the whole engine halts and waits for the file to be done then resume.

## Problem
We should make it possbile for the game to still run while assets are loading

## Options / solutions?
1. Current solution engine halts untill everything is loaded
2. have an `on_load_render()` and `on_load_update()`
3. have the asset ignored untill it is loaded

## Issues with these solutions
1. akward weird allows for no loading screen and little control over loading behavoir
2. makes state managment akward and abrubtly splits code
3. a bit strange but pushes work on the user both good and bad allows for more controll