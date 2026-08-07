package crypto

import (
	"bytes"
	"testing"
)

func TestEncryptDecrypt(t *testing.T) {
	key := []byte("test_encryption_key_32_byte_long")
	plaintext := []byte("secret password")

	encrypted, err := Encrypt(plaintext, key)
	if err != nil {
		t.Fatalf("Encrypt() error = %v", err)
	}

	if bytes.Equal(encrypted, plaintext) {
		t.Error("Encrypted data should not equal plaintext")
	}

	decrypted, err := Decrypt(encrypted, key)
	if err != nil {
		t.Fatalf("Decrypt() error = %v", err)
	}

	if !bytes.Equal(decrypted, plaintext) {
		t.Errorf("Decrypt() = %v, want %v", decrypted, plaintext)
	}
}

func TestEncryptDifferentOutputs(t *testing.T) {
	key := []byte("test_encryption_key_32_byte_long")
	plaintext := []byte("secret password")

	encrypted1, _ := Encrypt(plaintext, key)
	encrypted2, _ := Encrypt(plaintext, key)

	if bytes.Equal(encrypted1, encrypted2) {
		t.Error("Same plaintext should produce different ciphertext (random nonce)")
	}
}

func TestDecryptWrongKey(t *testing.T) {
	key1 := []byte("test_encryption_key_32_byte_long")
	key2 := []byte("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
	plaintext := []byte("secret password")

	encrypted, _ := Encrypt(plaintext, key1)
	_, err := Decrypt(encrypted, key2)

	if err == nil {
		t.Error("Decrypt with wrong key should fail")
	}
}

func TestEncryptInvalidKeyLength(t *testing.T) {
	shortKey := []byte("short_key")
	plaintext := []byte("secret")

	_, err := Encrypt(plaintext, shortKey)
	if err == nil {
		t.Error("Encrypt with invalid key length should fail")
	}
}
