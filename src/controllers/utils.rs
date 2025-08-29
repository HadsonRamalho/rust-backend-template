use std::env;

use axum::Json;
use dotenvy::dotenv;
use hyper::StatusCode;
use pwhash::bcrypt;
use rand::Rng;

use crate::models::error::ApiError;

pub fn validate_cpf(cpf: &str) -> bool {
    let cpf: Vec<u8> = cpf
        .chars()
        .filter(|c| c.is_digit(10))
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();

    if cpf.len() != 11 || cpf.iter().all(|&d| d == cpf[0]) {
        return false;
    }

    let soma1: u32 = cpf
        .iter()
        .take(9)
        .enumerate()
        .map(|(i, &d)| (10 - i as u32) * d as u32)
        .sum();

    let dig1 = if soma1 % 11 < 2 { 0 } else { 11 - (soma1 % 11) };

    let soma2: u32 = cpf
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, &d)| (11 - i as u32) * d as u32)
        .sum();

    let dig2 = if soma2 % 11 < 2 { 0 } else { 11 - (soma2 % 11) };

    cpf[9] == dig1 as u8 && cpf[10] == dig2 as u8
}

pub fn validate_cnpj(cnpj: &str) -> bool {
    let cnpj: Vec<u8> = cnpj
        .chars()
        .filter(|c| c.is_digit(10))
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();

    if cnpj.len() != 14 || cnpj.windows(2).all(|w| w[0] == w[1]) {
        return false;
    }

    let calc_digito = |slice: &[u8], pesos: &[u8]| -> u8 {
        let soma: u32 = slice
            .iter()
            .zip(pesos.iter())
            .map(|(&d, &p)| (d as u32) * (p as u32))
            .sum();
        let resto = soma % 11;
        if resto < 2 { 0 } else { (11 - resto) as u8 }
    };

    let pesos1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let pesos2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];

    let d1 = calc_digito(&cnpj[0..12], &pesos1);
    let d2 = calc_digito(&[&cnpj[0..12], &[d1]].concat(), &pesos2);

    cnpj[12] == d1 && cnpj[13] == d2
}

pub fn format_cnpj(cnpj: &str) -> Result<String, String> {
    let cnpj_numeros: Vec<char> = cnpj.chars().filter(|c: &char| c.is_ascii_digit()).collect();
    if cnpj_numeros.len() != 14 {
        return Err("Invalid CNPJ length".to_string());
    }
    let mut cnpj: Vec<char> = cnpj_numeros;
    cnpj.insert(2, '.');
    cnpj.insert(6, '.');
    cnpj.insert(10, '/');
    cnpj.insert(15, '-');
    let mut cnpjfinal: String = "".to_string();
    for u in cnpj {
        cnpjfinal.push(u);
    }
    Ok(cnpjfinal)
}

pub fn format_cpf(cpf: &str) -> Result<String, String> {
    let cpf: Vec<char> = cpf.chars().filter(|c: &char| c.is_digit(10)).collect();
    if cpf.len() != 11 {
        return Err("Invalid CPF length".to_string());
    }
    let mut cpf: Vec<char> = cpf;
    cpf.insert(3, '.');
    cpf.insert(7, '.');
    cpf.insert(11, '-');
    let mut cpffinal: String = "".to_string();
    for u in cpf {
        cpffinal.push(u);
    }
    Ok(cpffinal)
}

pub fn format_document(documento_: &str) -> Result<String, String> {
    if let Ok(cpf) = format_cpf(documento_) {
        return Ok(cpf);
    }
    if let Ok(cnpj) = format_cnpj(documento_) {
        return Ok(cnpj);
    }
    Err("Invalid document".to_string())
}

pub fn random_hash() -> String {
    let now = chrono::Utc::now().to_string();
    bcrypt::hash(now).unwrap()
}

pub fn password_hash(input: &str) -> String {
    let output = bcrypt::hash(input).unwrap();
    output
}

pub fn random_public_id() -> i32 {
    let output = rand::thread_rng().gen_range(1000000..9999999);
    output
}

pub fn get_database_url_from_env() -> Result<String, (StatusCode, Json<String>)> {
    dotenv().ok();

    match env::var("DATABASE_URL") {
        Ok(secret) => Ok(secret),
        Err(error) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::DatabaseConnection(error.to_string()).to_string()),
            ));
        }
    }
}

pub fn get_frontend_url_from_env() -> Result<String, (StatusCode, Json<String>)> {
    dotenv().ok();

    match env::var("FRONTEND_URL") {
        Ok(secret) => Ok(secret),
        Err(_) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::FrontendUrl.to_string()),
            ));
        }
    }
}

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::deadpool::{Object, Pool},
};

pub async fn get_conn(
    pool: &Pool<AsyncPgConnection>,
) -> Result<Object<AsyncPgConnection>, (StatusCode, Json<String>)> {
    pool.get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))
}
