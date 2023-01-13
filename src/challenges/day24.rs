use clap::Parser;
use pathfinding::directed::astar;
use std::{fmt, fs, str};

const COLLIDES_FLAG: u8 = 0b1000_0000u8;
const NORTH_FLAG: u8 = 0b0000_0001u8;
const EAST_FLAG: u8 = 0b0000_0010u8;
const SOUTH_FLAG: u8 = 0b0000_0100u8;
const WEST_FLAG: u8 = 0b0000_1000u8;

struct ObstacleType {
    tile_char: char,
    tile_flags: u8,
}

const EMPTY: ObstacleType = ObstacleType {
    tile_char: '.',
    tile_flags: 0x00,
};
const WALL: ObstacleType = ObstacleType {
    tile_char: '#',
    tile_flags: COLLIDES_FLAG,
};
const NORTH: ObstacleType = ObstacleType {
    tile_char: '^',
    tile_flags: COLLIDES_FLAG | NORTH_FLAG,
};
const EAST: ObstacleType = ObstacleType {
    tile_char: '>',
    tile_flags: COLLIDES_FLAG | EAST_FLAG,
};
const SOUTH: ObstacleType = ObstacleType {
    tile_char: 'v',
    tile_flags: COLLIDES_FLAG | SOUTH_FLAG,
};
const WEST: ObstacleType = ObstacleType {
    tile_char: '<',
    tile_flags: COLLIDES_FLAG | WEST_FLAG,
};

const PARSABLE: [ObstacleType; 6] = [EMPTY, WALL, NORTH, EAST, SOUTH, WEST];

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Point {
    x: usize,
    y: usize,
    t: usize,
}

impl Point {
    fn moves(&self) -> Vec<(Point, usize)> {
        let mut m = vec![
            Point {
                x: self.x,
                y: self.y,
                t: self.t + 1,
            },
            Point {
                x: self.x + 1,
                y: self.y,
                t: self.t + 1,
            },
            Point {
                x: self.x,
                y: self.y + 1,
                t: self.t + 1,
            },
        ];
        if self.x > 0 {
            m.push(Point {
                x: self.x - 1,
                y: self.y,
                t: self.t + 1,
            })
        }
        if self.y > 0 {
            m.push(Point {
                x: self.x,
                y: self.y - 1,
                t: self.t + 1,
            })
        }
        m.into_iter().map(|p| (p, 1)).collect()
    }

    fn distance(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

fn find_first_empty(line: &str) -> Result<usize, &str> {
    for (i, c) in line.chars().enumerate() {
        if c == '.' {
            return Ok(i);
        }
    }
    return Err("No blanks in line");
}

#[derive(Debug)]
struct Level {
    tiles: Vec<u8>,
    width: usize,
    height: usize,
    enter: Point,
    exit: Point,
}

impl Level {
    fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![0x00; width * height],
            width,
            height,
            enter: Point { x: 0, y: 0, t: 0 },
            exit: Point { x: 0, y: 0, t: 0 },
        }
    }

    fn future_tile(&self, x: usize, y: usize, t: usize) -> u8 {
        if x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1 {
            // walls are not affected by the temporal dimension
            return self.get_tile(x, y);
        }
        // transform coordinates for inner walls
        let u = x - 1;
        let v = y - 1;
        let inner_width = self.width - 2;
        let inner_height = self.height - 2;
        // get the offset for the point's time dimension
        let u_mod = t % inner_width;
        let v_mod = t % inner_height;
        // calculate coordinates for each direction according to the offsets
        // and switch back to the original coordinate system
        let north_y = ((inner_height + inner_height + v + v_mod) % inner_height);
        let south_y = ((inner_height + inner_height + v - v_mod) % inner_height);
        let east_x = ((inner_width + inner_width + u - u_mod) % inner_width);
        let west_x = ((inner_width + inner_width + u + u_mod) % inner_width);
        // get the tile data
        let south_bits = self.get_tile(x, south_y + 1) & SOUTH_FLAG;
        let north_bits = self.get_tile(x, north_y + 1) & NORTH_FLAG;
        let west_bits = self.get_tile(west_x + 1, y) & WEST_FLAG;
        let east_bits = self.get_tile(east_x + 1, y) & EAST_FLAG;
        let tile = north_bits | east_bits | south_bits | west_bits;
        // let tile = west_bits;
        if tile > 0 {
            return tile | COLLIDES_FLAG;
        } else {
            return 0x00;
        }
    }

    fn validate_point(&self, point: &Point) -> bool {
        let tile = self.future_tile(point.x, point.y, point.t);
        tile & COLLIDES_FLAG != COLLIDES_FLAG
    }

    fn is_exit_point(&self, point: &Point) -> bool {
        point.distance(&self.exit) == 0
    }

    fn successors(&self, point: &Point) -> Vec<(Point, usize)> {
        let moves: Vec<(Point, usize)> = point.moves();
        moves
            .into_iter()
            .filter(|p| self.validate_point(&p.0))
            .collect::<Vec<(Point, usize)>>()
    }

    fn get_tile(&self, x: usize, y: usize) -> u8 {
        self.tiles[(y * self.width) + x]
    }

    fn set_tile(&mut self, x: usize, y: usize, val: u8) {
        self.tiles[(y * self.width) + x] = val;
    }

    fn at(&self, t: usize) -> Self {
        let mut state = Self::new(self.width, self.height);
        state.enter = self.enter.clone();
        state.exit = self.exit.clone();
        for x in 0..(self.width) {
            for y in 0..(self.height) {
                state.set_tile(x, y, self.future_tile(x, y, t))
            }
        }
        state
    }

    fn solve(&self) -> Option<(Vec<Point>, usize)> {
        astar::astar(
            &self.enter,
            |p| self.successors(p),
            |p| self.exit.distance(p),
            |p| self.is_exit_point(p),
        )
    }

    fn to_ascii(&self) -> Vec<char> {
        let mut buf: Vec<char> = vec!['\0'; self.width * self.height + self.height];
        let mut offset: usize = 0;
        for (i, tile) in self.tiles.iter().enumerate() {
            let str_i = i + offset;
            buf[str_i] = '.';
            for obstacle_type in PARSABLE {
                if tile & obstacle_type.tile_flags == obstacle_type.tile_flags {
                    match buf[str_i] {
                        '.' | '#' => buf[str_i] = obstacle_type.tile_char,
                        '^' | '>' | 'V' | '<' => buf[str_i] = '2',
                        '2' => buf[str_i] = '3',
                        '3' => buf[str_i] = '4',
                        _ => buf[str_i] = '?',
                    }
                }
            }
            if i > 0 && (i + 1) % self.width == 0 && i < self.tiles.len() - 1 {
                buf[str_i + 1] = '\n';
                offset += 1;
            }
        }
        buf
    }

    fn print_solution(&self) {
        let (path, cost) = self.solve().unwrap();
        for (i, point) in path.iter().enumerate() {
            let mut ascii = self.at(i).to_ascii();
            ascii[point.y * (self.width + 1) + point.x] = 'E';
            let s = ascii.iter().collect::<String>();
            println!("\n@ t={}, {:?}\n{}", i, point, s);
        }
    }

    fn parse(path: &String) -> Self {
        let input = fs::read_to_string(path).expect("I/O error");
        let mut width: usize = 0;
        let mut height: usize = 0;
        for line in input.lines() {
            height += 1;
            if line.len() > width {
                width = line.len();
            }
        }
        let mut state = Self::new(width, height);
        for (y, line) in input.lines().enumerate() {
            if y == 0 {
                let start_x = find_first_empty(line).unwrap();
                state.enter.x = start_x;
            }
            if y == height - 1 {
                let exit_x = find_first_empty(line).unwrap();
                state.exit.x = exit_x;
                state.exit.y = y;
            }
            for (x, c) in line.chars().enumerate() {
                for obstacle_type in PARSABLE {
                    if c == obstacle_type.tile_char {
                        state.set_tile(x, y, obstacle_type.tile_flags);
                        break;
                    }
                }
            }
        }
        state
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.to_ascii().iter().collect::<String>();
        write!(f, "{}", s)
    }
}

#[derive(clap::Args, Debug)]
pub struct Args {
    #[arg(default_value_t = String::from("./inputs/day24/input.txt"))]
    file: String,
}

pub fn entrypoint(args: &Args) {
    let level = Level::parse(&args.file);
    println!("Loaded map:\n{}", level);
    println!("\nStart {:?}, Goal {:?}", level.enter, level.exit);
    level.print_solution();
}
