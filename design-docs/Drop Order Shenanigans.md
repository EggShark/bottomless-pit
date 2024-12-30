# Drop Order basics
In rust the drop order is decided in the same order fields are defined on a struct for example:
```rust
struct Example {
    x: T,
    y: T,
    z: T,
}
```
x will be dropped first then y and finally z. On a tuple it works much the same way with the first elment of the tuple being dropped first then the second

# Why does this matter for BP
For some reason only Wayland seems to have an issue with drop order but other desktop mangaers and OS would likley have problems anyway. On wayland there was an "inexplicable" segmentation fault when closing the window. As explained to me by someone in this [issue](https://github.com/rust-windowing/winit/issues/4040) I was not dropping all my graphics objects before dropping the winit window.

## The Solution
While I thought I tried everything aka making the graphics context drop last and making the window drop last I missed one important thing in the `Application Handler` trait which is this signature:
```rust
impl<T: Game> ApplicationHandler<BpEvent> for (Engine, T)
```
As mentioned above this would drop the engine first (including the window) and then the users data. The user stores things like buffers and bindgroups so of course this would cause an issue when drop orders begin to matter. The solution is quite simple just switch the Engine and T around to:
```rust
impl<T: Game> ApplicationHandler<BpEvent> for (T, Engine)
```
Now Everything works !
So **MAKE SURE THE WINDOW IS ALWAYS DROPPED LAST**