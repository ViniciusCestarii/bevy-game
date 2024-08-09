# JWT

JSON Web Token (JWT)

## What's it?

The **JSON Web Token** specification is an industry standard to represent claims securely between two parties. The **JWT** is a ```base64``` encoded JSON object that contains key-value pairs of attributes that are signed by a trusted authority.

## Best Practices

A **JWT** carries information that your end-users pass to the system to be recognized as legitimate users, along with other metadata. To keep the token payload secure and straightforward, consider the following:

```ts
interface JWTBody {
  sub: string // the user identifier
  name: string
  avatarUrl: string
}
```

❌ Don't add confidetial info
  - NEVER add passwords
  - Evaluate if you really need to pass email

✔ Use Cookie instead of Authorization header storage in localstorage

Why?

- Security:
  - Storing JWTs in cookies is generally safer than localStorage, especially if you set the following attributes on the cookie:
    - Secure: Ensures the cookie is only sent over HTTPS.
    - HttpOnly: Prevents JavaScript access to the cookie, mitigating XSS attacks.
    - SameSite=Strict: Prevents the cookie from being sent along with cross-site requests, reducing CSRF risks.

- Availability:
  - Cookies are automatically included in every HTTP request made to the same origin, ensuring the token is always available when needed.
