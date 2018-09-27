extern crate csv;
#[macro_use]
extern crate failure;
extern crate image;
extern crate shared;
#[macro_use]
extern crate lazy_static;

mod algebraic_notation;

use image::{ImageBuffer, ImageDecoder, Rgb, Rgba};
use shared::enum_primitive::FromPrimitive;
use shared::{BoardState, Piece, Result};
use std::collections::HashMap;
use std::fs::{self, File};

fn main() {
    let input_file = "games.csv";
    let requested_id = std::env::args().nth(1);

    let _ = fs::remove_dir_all("board_states");

    let mut parser = csv::Reader::from_path(input_file).expect("Could not open games.csv");
    for record in parser.records() {
        let record = record.expect("Could not parser row");

        let game_id = record.get(0).unwrap();
        if let Some(requested_id) = requested_id.as_ref() {
            if requested_id != game_id {
                continue;
            }
        }
        let moves = record.get(12).unwrap();
        println!("Generating game {:?}", game_id);

        fs::create_dir_all(&format!("board_states/{}", game_id))
            .expect("Could not create directory");

        let mut boardstate = BoardState::init();

        generate_image(&boardstate, &format!("board_states/{}/{}.png", game_id, 0))
            .expect("Cannot generate image");

        for (index, m) in moves.split(' ').enumerate() {
            // println!("{:?}: {:?} {}", index + 1, boardstate.current_player, m);
            let _notation = algebraic_notation::Notation::parse(m).expect("Could not parse");
            /*notation
                .apply(&mut boardstate)
                .expect("Could not apply notation");
            /*if let Err(e) = boardstate.make_move(m) {
                println!("{:?}", e);
                break;
            }*/
            generate_image(
            &boardstate,
            &format!("board_states/{}/{}.png", game_id, index + 1),
            )
            .expect("Cannot generate image");*/
        }
        println!("Done generating game {:?}", game_id);
    }
}

const PIXEL_DARK_GRAY: Rgb<u8> = Rgb { data: [50, 50, 50] };
const PIXEL_LIGHT_GRAY: Rgb<u8> = Rgb {
    data: [200, 200, 200],
};

lazy_static! {
    static ref BOARD_IMAGE: ImageBuffer<Rgb<u8>, Vec<u8>> = {
        let container = vec![0; 3 * 400 * 400];
        let mut image = ImageBuffer::<Rgb<u8>, _>::from_raw(400, 400, container)
            .expect("Board ImageBuffer not big enough");
        for x in 0..8 {
            for y in 0..8 {
                for w in 0..50 {
                    for h in 0..50 {
                        let color = if (x + y) % 2 == 0 {
                            PIXEL_LIGHT_GRAY
                        } else {
                            PIXEL_DARK_GRAY
                        };
                        let x = x * 50 + w;
                        let y = y * 50 + h;
                        image.put_pixel(x, y, color);
                    }
                }
            }
        }
        image
    };
}

fn generate_image(state: &BoardState, out: &str) -> Result<()> {
    let mut board = BOARD_IMAGE.clone();
    for x in 0..8 {
        for y in 0..8 {
            let piece = state.get_piece(x, y);
            if piece != Piece::None {
                let image = get_image(piece);
                for w in 0..50 {
                    for h in 0..50 {
                        let pixel = *image.get_pixel(w, h);
                        if pixel.data[3] > 150 {
                            board.put_pixel(
                                u32::from(x) * 50 + w,
                                u32::from(y) * 50 + h,
                                Rgb {
                                    data: [pixel.data[0], pixel.data[1], pixel.data[2]],
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    board.save(out)?;
    Ok(())
}

lazy_static! {
    static ref PIECE_SPRITES: HashMap<Piece, ImageBuffer<Rgba<u8>, Vec<u8>>> = {
        let mut map = HashMap::new();
        for piece in Piece::WhiteKing as u8..=Piece::BlackRookMoved as u8 {
            let url = match Piece::from_u8(piece).unwrap() {
                Piece::WhiteKing | Piece::WhiteKingMoved => "visualiser/sprites/whiteKing.png",
                Piece::WhiteRook | Piece::WhiteRookMoved => "visualiser/sprites/whiteRook.png",
                Piece::WhiteBishop => "visualiser/sprites/whiteBishop.png",
                Piece::WhiteKnight => "visualiser/sprites/whiteKnight.png",
                Piece::WhiteQueen => "visualiser/sprites/whiteQueen.png",
                Piece::WhitePawn | Piece::WhitePawnMoved => "visualiser/sprites/whitePawn.png",

                Piece::BlackKing | Piece::BlackKingMoved => "visualiser/sprites/blackKing.png",
                Piece::BlackRook | Piece::BlackRookMoved => "visualiser/sprites/blackRook.png",
                Piece::BlackBishop => "visualiser/sprites/blackBishop.png",
                Piece::BlackKnight => "visualiser/sprites/blackKnight.png",
                Piece::BlackQueen => "visualiser/sprites/blackQueen.png",
                Piece::BlackPawn | Piece::BlackPawnMoved => "visualiser/sprites/blackPawn.png",
                Piece::None => unreachable!(),
            };
            let mut image = image::png::PNGDecoder::new(File::open(url).unwrap());
            assert_eq!(image::RGBA(8), image.colortype().unwrap());
            let image = image.read_image().unwrap();
            let image = match image {
                image::DecodingResult::U8(b) => b,
                _ => unimplemented!(),
            };

            let buffer = vec![0u8; 50 * 50 * 4];
            let mut buffer = ImageBuffer::from_raw(50, 50, buffer).unwrap();

            for x in 0..50 {
                for y in 0..50 {
                    let offset = x * 4 + y * 50 * 4;
                    let rgba = Rgba {
                        data: [
                            image[offset],
                            image[offset + 1],
                            image[offset + 2],
                            image[offset + 3],
                        ],
                    };
                    buffer.put_pixel(x as u32, y as u32, rgba);
                }
            }
            map.insert(Piece::from_u8(piece).unwrap(), buffer);
        }
        map
    };
}

fn get_image(piece: Piece) -> &'static ImageBuffer<Rgba<u8>, Vec<u8>> {
    PIECE_SPRITES.get(&piece).unwrap()
}
