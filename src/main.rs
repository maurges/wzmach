mod input_event;

fn main() {
    let producer = input_event::GestureProducer::new();
    for event in producer {
        println!("update: {:?}", event);
    }
}

