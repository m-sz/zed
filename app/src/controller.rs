pub struct Button {
    pressed: bool,
    previous: bool,
}

pub struct Direction {
    x: f64,
    y: f64
}

pub struct Controller {
    pub left: Button,
    pub right: Button,
    pub up: Button,
    pub down: Button,
    pub direction: Direction
}

impl Button {
    pub fn new() -> Self {
        Self {
            pressed: false,
            previous: false
        }
    }

    pub fn update(&mut self, pressed: bool) {
        self.previous = self.pressed;
        self.pressed = pressed;
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn rising(&self) -> bool {
        !self.previous && self.pressed
    }

    pub fn falling(&self) -> bool {
        self.previous && !self.pressed
    }
}

impl Direction {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y
        }
    }

    pub fn direction(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

impl Controller {
    pub fn new() -> Self {
        Self {
            left: Button::new(),
            right: Button::new(),
            up: Button::new(),
            down: Button::new(),
            direction: Direction::new(0.0, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Button;
    #[test]
    fn test_button_update() {
        let mut button = Button::new();

        assert_eq!(button.pressed(), false, "Initial state of a button is not pressed");
        button.update(true);
        assert_eq!(button.pressed(), true);
    }

    #[test]
    fn test_button_rising_falling_edges() {
        let mut button = Button::new();

        button.update(true);
        assert_eq!(button.rising(), true);

        button.update(true);
        assert_eq!(button.rising(), false, "Rising edge is detected only after button has been updated from 'false' to 'true'");

        button.update(true);
        assert_eq!(button.rising(), false);

        button.update(false);
        assert_eq!(button.falling(), true);

        button.update(false);
        assert_eq!(button.falling(), false);

        button.update(false);
        assert_eq!(button.falling(), false);

        button.update(true);
        assert_eq!(button.rising(), true);

        button.update(false);
        assert_eq!(button.falling(), true);
    }
}
