pub mod rendering;

/// This module contains functions related to rendering graphics.
mod graphics {
    // Calculates the total area of a rectangle given its width and height
    fn calculate_area(width: f32, height: f32) -> f32 {
        width * height
    }

    // Resizes a rectangle to a new width and height
    fn resize_rectangle(rect: &mut Rectangle, new_width: f32, new_height: f32) {
        rect.width = new_width;
        rect.height = new_height;
    }
}

/// A struct representing a rectangle
struct Rectangle {
    width: f32,
    height: f32,
}

/// Main rendering function
fn render() {
    let rect = Rectangle {
        width: 10.0,
        height: 20.0,
    };

    let area = graphics::calculate_area(rect.width, rect.height);
    println!("Area of rectangle: {}", area);

    graphics::resize_rectangle(&mut rect, 5.0, 10.0);
    println!("Resized rectangle: {} x {}", rect.width, rect.height);
}

// Entry point of the program
fn main() {
    render();
}