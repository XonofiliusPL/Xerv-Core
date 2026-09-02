use xerv_core::logging;

#[test]
fn init_does_not_panic() {
    // W testach integracyjnych inny test mógł już zainicjować subscriber;
    // `try_init` zwraca wtedy błąd — akceptujemy oba wyniki.
    let _ = logging::init("debug");
}
