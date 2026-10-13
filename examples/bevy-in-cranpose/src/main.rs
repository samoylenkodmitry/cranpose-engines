fn main() -> Result<(), cranpose::LaunchError> {
    bevy_in_cranpose::launcher().try_run(bevy_in_cranpose::Root)
}
