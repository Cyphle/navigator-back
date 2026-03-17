// TODO c'est pas besoin cette gestion d'erreur centralisée parce que tout le monde doit traiter tous les cas
// Il faut réfléchir à un générique et que les autres étendent ou alors on s'en fout
#[derive(Debug)]
pub enum ApplicationErrors {
    FamilyAlreadyExists,
    MissingUsername,
    Database(sqlx::Error),
}
