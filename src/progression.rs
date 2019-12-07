
//messages sent by other thread to specify their progression
pub enum Command {
    Read(Place, i32, i32, i32), //src/dst nb_path_done  nb_fold_in_current_path nb_fic_in_current_path
    join(i32, i32, i32),
    RemoveFile(PathBuf),
    RemoveFold(PathBuf, u32, u32),
}