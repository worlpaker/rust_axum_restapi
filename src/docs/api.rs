use crate::library;
use crate::library_web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Library API",
        version = "0.1.0",
        description = "API for managing library book rentals."
    ),
    paths(

        //author
        library_web::author::create_author,
        library_web::author::authors,
        library_web::author::get_author,

        //book
        library_web::book::create_book,
        library_web::book::books,
        library_web::book::get_book,

        //user
        library_web::user::create_user,
        library_web::user::rent_book,
        library_web::user::users,
        library_web::user::get_user,

    ),
    components(schemas(
    
        //author
        library::author::Author,
        library::author::AuthorRow,
        library_web::author::CreatedAuthorBody,
        library_web::author::AuthorsBody,
        library_web::author::GetAuthorBody,

        //book
        library::book::Book,
        library::book::Status,
        library_web::book::CreatedBookBody,
        library_web::book::BooksBody,
        library_web::book::GetBookBody,
    
        //user
        library::user::User,
        library::user::RentBook,
        library::user::UserRow,
        library::user::UserRentBook,
        library::user::UserHistoryRow,
        library_web::user::CreatedUserBody,
        library_web::user::RentedBookBody,
        library_web::user::UsersBody,
        library_web::user::GetUserBody,

        ),
    ),
   
)]
pub struct ApiDoc;
