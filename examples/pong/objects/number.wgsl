#mod main
#import objects.digit.main

const DIGIT_COUNT_PER_NUMBER = 2;
const SEGMENT_COUNT_PER_NUMBER = DIGIT_COUNT_PER_NUMBER * SEGMENT_COUNT_PER_DIGIT;

struct Number {
    value: u32,
    segments: array<DigitSegment, SEGMENT_COUNT_PER_NUMBER>,
}

#mod compute
#import ~.main
#import objects.digit.compute

const NUMBER_HEIGHT = 0.18;
const NUMBER_SPACING_X = 0.14;

fn init_number(position: vec3f, value: u32) -> Number {
    var number = Number(value, array<DigitSegment, SEGMENT_COUNT_PER_NUMBER>());
    for (var digit_index = 0u; digit_index < DIGIT_COUNT_PER_NUMBER; digit_index++) {
        let digit = _number_digit(digit_index, value, position);
        for (var segment_index = 0u; segment_index < SEGMENT_COUNT_PER_DIGIT; segment_index++) {
            let absolute_segment_index = digit_index * SEGMENT_COUNT_PER_DIGIT + segment_index;
            number.segments[absolute_segment_index] = digit.segments[segment_index];
        }
    }
    return number;
}

fn _number_digit(index: u32, number_value: u32, number_position: vec3f) -> Digit {
    const middle_digit_index = f32(DIGIT_COUNT_PER_NUMBER) / 2.;
    let position = vec3f(
        number_position.x + (f32(index) + 0.5 - middle_digit_index) * NUMBER_SPACING_X,
        number_position.y,
        number_position.z + 0.01 * (DIGIT_COUNT_PER_NUMBER - f32(index))
    );
    let value = u32(f32(number_value) / pow(10, f32(DIGIT_COUNT_PER_NUMBER - index - 1))) % 10;
    return init_digit(position, NUMBER_HEIGHT, value);
}
