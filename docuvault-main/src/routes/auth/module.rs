pub mod password{
    use argon2::{Argon2, password_hash::SaltString, PasswordHasher, PasswordHash, PasswordVerifier};
    use rand::rngs::OsRng;
    
    pub fn create_hash(password: &[u8]) -> Result<String, argon2::password_hash::Error>{
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password, &salt)?.to_string();
        Ok(password_hash)
    }
    pub fn verify_password(hash: &str, password: &[u8]) -> Result<(), argon2::password_hash::Error>{
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        argon2.verify_password(password, &parsed_hash)?;
        Ok(())
    }
}
