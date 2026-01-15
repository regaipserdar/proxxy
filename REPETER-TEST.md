### 1. Tüm Postları Çekme (GET)
```http
GET /posts HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```

### 2. Tek Bir Kullanıcıyı Çekme (GET - ID: 1)
```http
GET /users/1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```

### 3. Yeni Post Oluşturma (POST)
```http
POST /posts HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Content-Type: application/json
Accept: application/json

{
  "title": "Deneme Baslik",
  "body": "Bu bir mock post istegidir.",
  "userId": 1
}
```

### 4. Post Güncelleme - Tam Değişim (PUT)
```http
PUT /posts/1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Content-Type: application/json
Accept: application/json

{
  "id": 1,
  "title": "Guncellenmis Baslik",
  "body": "Icerik tamamen degistirildi.",
  "userId": 1
}
```

### 5. Post Güncelleme - Kısmi Değişim (PATCH)
```http
PATCH /posts/1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Content-Type: application/json
Accept: application/json

{
  "title": "Sadece Baslik Degisti"
}
```

### 6. Post Silme (DELETE)
```http
DELETE /posts/1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: */*
```

### 7. Filtreleme ile Yorumları Çekme (Query Params)
```http
GET /comments?postId=1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```

### 8. İç İçe Kaynak (Nested Resource - Kullanıcının Todoları)
```http
GET /users/1/todos HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```

### 9. Fotoğrafları Çekme (Albüm ID'ye göre)
```http
GET /photos?albumId=1 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```

### 10. Olmayan Bir Kaynak (404 Testi)
```http
GET /posts/99999 HTTP/1.1
Host: jsonplaceholder.typicode.com
User-Agent: Test-Client/1.0
Accept: application/json
```