//! HTTP status codes.

use std::fmt;
use std::mem;
use std::cmp::Ordering;

pub use self::StatusCode::*;
pub use self::StatusClass::*;

/// An HTTP status code (`status-code` in RFC 7230 et al.).
///
/// This enum is absolutely exhaustive, covering all 500 possible values (100–599).
///
/// As this is a C‐style enum with each variant having a corresponding value, you may use the likes
/// of `Continue as u16` to retreive the value `100u16`. Normally, though, you should not need to
/// do any such thing; just use the status code as a `StatusCode`.
///
/// If you encounter a status code that you do not know how to deal with, you should treat it as
/// the `x00` status code—e.g. for code 123, treat it as 100 (Continue). This can be achieved with
/// `self.class().default_code()`:
///
/// ```rust
/// # use teepee::status::{Code123, Continue};
/// assert_eq!(Code123.class().default_code(), Continue);
/// ```
///
/// IANA maintain the [Hypertext Transfer Protocol (HTTP) Status Code
/// Registry](http://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml) which is
/// the source for this enum (with one exception, 418 I'm a teapot, which is inexplicably not
/// in the register).
pub enum StatusCode {
    /// 100 Continue
    /// [[RFC7231, Section 6.2.1](https://tools.ietf.org/html/rfc7231#section-6.2.1)]
    Continue = 100,
    /// 101 Switching Protocols
    /// [[RFC7231, Section 6.2.2](https://tools.ietf.org/html/rfc7231#section-6.2.2)]
    SwitchingProtocols = 101,
    /// 102 Processing
    /// [[RFC2518](https://tools.ietf.org/html/rfc2518)]
    Processing = 102,
    /// 103 (unregistered)
    Code103 = 103,
    /// 104 (unregistered)
    Code104 = 104,
    /// 105 (unregistered)
    Code105 = 105,
    /// 106 (unregistered)
    Code106 = 106,
    /// 107 (unregistered)
    Code107 = 107,
    /// 108 (unregistered)
    Code108 = 108,
    /// 109 (unregistered)
    Code109 = 109,
    /// 110 (unregistered)
    Code110 = 110,
    /// 111 (unregistered)
    Code111 = 111,
    /// 112 (unregistered)
    Code112 = 112,
    /// 113 (unregistered)
    Code113 = 113,
    /// 114 (unregistered)
    Code114 = 114,
    /// 115 (unregistered)
    Code115 = 115,
    /// 116 (unregistered)
    Code116 = 116,
    /// 117 (unregistered)
    Code117 = 117,
    /// 118 (unregistered)
    Code118 = 118,
    /// 119 (unregistered)
    Code119 = 119,
    /// 120 (unregistered)
    Code120 = 120,
    /// 121 (unregistered)
    Code121 = 121,
    /// 122 (unregistered)
    Code122 = 122,
    /// 123 (unregistered)
    Code123 = 123,
    /// 124 (unregistered)
    Code124 = 124,
    /// 125 (unregistered)
    Code125 = 125,
    /// 126 (unregistered)
    Code126 = 126,
    /// 127 (unregistered)
    Code127 = 127,
    /// 128 (unregistered)
    Code128 = 128,
    /// 129 (unregistered)
    Code129 = 129,
    /// 130 (unregistered)
    Code130 = 130,
    /// 131 (unregistered)
    Code131 = 131,
    /// 132 (unregistered)
    Code132 = 132,
    /// 133 (unregistered)
    Code133 = 133,
    /// 134 (unregistered)
    Code134 = 134,
    /// 135 (unregistered)
    Code135 = 135,
    /// 136 (unregistered)
    Code136 = 136,
    /// 137 (unregistered)
    Code137 = 137,
    /// 138 (unregistered)
    Code138 = 138,
    /// 139 (unregistered)
    Code139 = 139,
    /// 140 (unregistered)
    Code140 = 140,
    /// 141 (unregistered)
    Code141 = 141,
    /// 142 (unregistered)
    Code142 = 142,
    /// 143 (unregistered)
    Code143 = 143,
    /// 144 (unregistered)
    Code144 = 144,
    /// 145 (unregistered)
    Code145 = 145,
    /// 146 (unregistered)
    Code146 = 146,
    /// 147 (unregistered)
    Code147 = 147,
    /// 148 (unregistered)
    Code148 = 148,
    /// 149 (unregistered)
    Code149 = 149,
    /// 150 (unregistered)
    Code150 = 150,
    /// 151 (unregistered)
    Code151 = 151,
    /// 152 (unregistered)
    Code152 = 152,
    /// 153 (unregistered)
    Code153 = 153,
    /// 154 (unregistered)
    Code154 = 154,
    /// 155 (unregistered)
    Code155 = 155,
    /// 156 (unregistered)
    Code156 = 156,
    /// 157 (unregistered)
    Code157 = 157,
    /// 158 (unregistered)
    Code158 = 158,
    /// 159 (unregistered)
    Code159 = 159,
    /// 160 (unregistered)
    Code160 = 160,
    /// 161 (unregistered)
    Code161 = 161,
    /// 162 (unregistered)
    Code162 = 162,
    /// 163 (unregistered)
    Code163 = 163,
    /// 164 (unregistered)
    Code164 = 164,
    /// 165 (unregistered)
    Code165 = 165,
    /// 166 (unregistered)
    Code166 = 166,
    /// 167 (unregistered)
    Code167 = 167,
    /// 168 (unregistered)
    Code168 = 168,
    /// 169 (unregistered)
    Code169 = 169,
    /// 170 (unregistered)
    Code170 = 170,
    /// 171 (unregistered)
    Code171 = 171,
    /// 172 (unregistered)
    Code172 = 172,
    /// 173 (unregistered)
    Code173 = 173,
    /// 174 (unregistered)
    Code174 = 174,
    /// 175 (unregistered)
    Code175 = 175,
    /// 176 (unregistered)
    Code176 = 176,
    /// 177 (unregistered)
    Code177 = 177,
    /// 178 (unregistered)
    Code178 = 178,
    /// 179 (unregistered)
    Code179 = 179,
    /// 180 (unregistered)
    Code180 = 180,
    /// 181 (unregistered)
    Code181 = 181,
    /// 182 (unregistered)
    Code182 = 182,
    /// 183 (unregistered)
    Code183 = 183,
    /// 184 (unregistered)
    Code184 = 184,
    /// 185 (unregistered)
    Code185 = 185,
    /// 186 (unregistered)
    Code186 = 186,
    /// 187 (unregistered)
    Code187 = 187,
    /// 188 (unregistered)
    Code188 = 188,
    /// 189 (unregistered)
    Code189 = 189,
    /// 190 (unregistered)
    Code190 = 190,
    /// 191 (unregistered)
    Code191 = 191,
    /// 192 (unregistered)
    Code192 = 192,
    /// 193 (unregistered)
    Code193 = 193,
    /// 194 (unregistered)
    Code194 = 194,
    /// 195 (unregistered)
    Code195 = 195,
    /// 196 (unregistered)
    Code196 = 196,
    /// 197 (unregistered)
    Code197 = 197,
    /// 198 (unregistered)
    Code198 = 198,
    /// 199 (unregistered)
    Code199 = 199,

    /// 200 OK
    /// [[RFC7231, Section 6.3.1](https://tools.ietf.org/html/rfc7231#section-6.3.1)]
    Ok = 200,
    /// 201 Created
    /// [[RFC7231, Section 6.3.2](https://tools.ietf.org/html/rfc7231#section-6.3.2)]
    Created = 201,
    /// 202 Accepted
    /// [[RFC7231, Section 6.3.3](https://tools.ietf.org/html/rfc7231#section-6.3.3)]
    Accepted = 202,
    /// 203 Non-Authoritative Information
    /// [[RFC7231, Section 6.3.4](https://tools.ietf.org/html/rfc7231#section-6.3.4)]
    NonAuthoritativeInformation = 203,
    /// 204 No Content
    /// [[RFC7231, Section 6.3.5](https://tools.ietf.org/html/rfc7231#section-6.3.5)]
    NoContent = 204,
    /// 205 Reset Content
    /// [[RFC7231, Section 6.3.6](https://tools.ietf.org/html/rfc7231#section-6.3.6)]
    ResetContent = 205,
    /// 206 Partial Content
    /// [[RFC7233, Section 4.1](https://tools.ietf.org/html/rfc7233#section-4.1)]
    PartialContent = 206,
    /// 207 Multi-Status
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    MultiStatus = 207,
    /// 208 Already Reported
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    AlreadyReported = 208,
    /// 209 (unregistered)
    Code209 = 209,
    /// 210 (unregistered)
    Code210 = 210,
    /// 211 (unregistered)
    Code211 = 211,
    /// 212 (unregistered)
    Code212 = 212,
    /// 213 (unregistered)
    Code213 = 213,
    /// 214 (unregistered)
    Code214 = 214,
    /// 215 (unregistered)
    Code215 = 215,
    /// 216 (unregistered)
    Code216 = 216,
    /// 217 (unregistered)
    Code217 = 217,
    /// 218 (unregistered)
    Code218 = 218,
    /// 219 (unregistered)
    Code219 = 219,
    /// 220 (unregistered)
    Code220 = 220,
    /// 221 (unregistered)
    Code221 = 221,
    /// 222 (unregistered)
    Code222 = 222,
    /// 223 (unregistered)
    Code223 = 223,
    /// 224 (unregistered)
    Code224 = 224,
    /// 225 (unregistered)
    Code225 = 225,
    /// 226 IM Used
    /// [[RFC3229](https://tools.ietf.org/html/rfc3229)]
    ImUsed = 226,
    /// 227 (unregistered)
    Code227 = 227,
    /// 228 (unregistered)
    Code228 = 228,
    /// 229 (unregistered)
    Code229 = 229,
    /// 230 (unregistered)
    Code230 = 230,
    /// 231 (unregistered)
    Code231 = 231,
    /// 232 (unregistered)
    Code232 = 232,
    /// 233 (unregistered)
    Code233 = 233,
    /// 234 (unregistered)
    Code234 = 234,
    /// 235 (unregistered)
    Code235 = 235,
    /// 236 (unregistered)
    Code236 = 236,
    /// 237 (unregistered)
    Code237 = 237,
    /// 238 (unregistered)
    Code238 = 238,
    /// 239 (unregistered)
    Code239 = 239,
    /// 240 (unregistered)
    Code240 = 240,
    /// 241 (unregistered)
    Code241 = 241,
    /// 242 (unregistered)
    Code242 = 242,
    /// 243 (unregistered)
    Code243 = 243,
    /// 244 (unregistered)
    Code244 = 244,
    /// 245 (unregistered)
    Code245 = 245,
    /// 246 (unregistered)
    Code246 = 246,
    /// 247 (unregistered)
    Code247 = 247,
    /// 248 (unregistered)
    Code248 = 248,
    /// 249 (unregistered)
    Code249 = 249,
    /// 250 (unregistered)
    Code250 = 250,
    /// 251 (unregistered)
    Code251 = 251,
    /// 252 (unregistered)
    Code252 = 252,
    /// 253 (unregistered)
    Code253 = 253,
    /// 254 (unregistered)
    Code254 = 254,
    /// 255 (unregistered)
    Code255 = 255,
    /// 256 (unregistered)
    Code256 = 256,
    /// 257 (unregistered)
    Code257 = 257,
    /// 258 (unregistered)
    Code258 = 258,
    /// 259 (unregistered)
    Code259 = 259,
    /// 260 (unregistered)
    Code260 = 260,
    /// 261 (unregistered)
    Code261 = 261,
    /// 262 (unregistered)
    Code262 = 262,
    /// 263 (unregistered)
    Code263 = 263,
    /// 264 (unregistered)
    Code264 = 264,
    /// 265 (unregistered)
    Code265 = 265,
    /// 266 (unregistered)
    Code266 = 266,
    /// 267 (unregistered)
    Code267 = 267,
    /// 268 (unregistered)
    Code268 = 268,
    /// 269 (unregistered)
    Code269 = 269,
    /// 270 (unregistered)
    Code270 = 270,
    /// 271 (unregistered)
    Code271 = 271,
    /// 272 (unregistered)
    Code272 = 272,
    /// 273 (unregistered)
    Code273 = 273,
    /// 274 (unregistered)
    Code274 = 274,
    /// 275 (unregistered)
    Code275 = 275,
    /// 276 (unregistered)
    Code276 = 276,
    /// 277 (unregistered)
    Code277 = 277,
    /// 278 (unregistered)
    Code278 = 278,
    /// 279 (unregistered)
    Code279 = 279,
    /// 280 (unregistered)
    Code280 = 280,
    /// 281 (unregistered)
    Code281 = 281,
    /// 282 (unregistered)
    Code282 = 282,
    /// 283 (unregistered)
    Code283 = 283,
    /// 284 (unregistered)
    Code284 = 284,
    /// 285 (unregistered)
    Code285 = 285,
    /// 286 (unregistered)
    Code286 = 286,
    /// 287 (unregistered)
    Code287 = 287,
    /// 288 (unregistered)
    Code288 = 288,
    /// 289 (unregistered)
    Code289 = 289,
    /// 290 (unregistered)
    Code290 = 290,
    /// 291 (unregistered)
    Code291 = 291,
    /// 292 (unregistered)
    Code292 = 292,
    /// 293 (unregistered)
    Code293 = 293,
    /// 294 (unregistered)
    Code294 = 294,
    /// 295 (unregistered)
    Code295 = 295,
    /// 296 (unregistered)
    Code296 = 296,
    /// 297 (unregistered)
    Code297 = 297,
    /// 298 (unregistered)
    Code298 = 298,
    /// 299 (unregistered)
    Code299 = 299,

    /// 300 Multiple Choices
    /// [[RFC7231, Section 6.4.1](https://tools.ietf.org/html/rfc7231#section-6.4.1)]
    MultipleChoices = 300,
    /// 301 Moved Permanently
    /// [[RFC7231, Section 6.4.2](https://tools.ietf.org/html/rfc7231#section-6.4.2)]
    MovedPermanently = 301,
    /// 302 Found
    /// [[RFC7231, Section 6.4.3](https://tools.ietf.org/html/rfc7231#section-6.4.3)]
    Found = 302,
    /// 303 See Other
    /// [[RFC7231, Section 6.4.4](https://tools.ietf.org/html/rfc7231#section-6.4.4)]
    SeeOther = 303,
    /// 304 Not Modified
    /// [[RFC7232, Section 4.1](https://tools.ietf.org/html/rfc7232#section-4.1)]
    NotModified = 304,
    /// 305 Use Proxy
    /// [[RFC7231, Section 6.4.5](https://tools.ietf.org/html/rfc7231#section-6.4.5)]
    UseProxy = 305,
    /// 306 (Unused)
    /// [[RFC7231, Section 6.4.6](https://tools.ietf.org/html/rfc7231#section-6.4.6)]
    Code306 = 306,
    /// 307 Temporary Redirect
    /// [[RFC7231, Section 6.4.7](https://tools.ietf.org/html/rfc7231#section-6.4.7)]
    TemporaryRedirect = 307,
    /// 308 Permanent Redirect
    /// [[RFC7238](https://tools.ietf.org/html/rfc7238)]
    PermanentRedirect = 308,
    /// 309 (unregistered)
    Code309 = 309,
    /// 310 (unregistered)
    Code310 = 310,
    /// 311 (unregistered)
    Code311 = 311,
    /// 312 (unregistered)
    Code312 = 312,
    /// 313 (unregistered)
    Code313 = 313,
    /// 314 (unregistered)
    Code314 = 314,
    /// 315 (unregistered)
    Code315 = 315,
    /// 316 (unregistered)
    Code316 = 316,
    /// 317 (unregistered)
    Code317 = 317,
    /// 318 (unregistered)
    Code318 = 318,
    /// 319 (unregistered)
    Code319 = 319,
    /// 320 (unregistered)
    Code320 = 320,
    /// 321 (unregistered)
    Code321 = 321,
    /// 322 (unregistered)
    Code322 = 322,
    /// 323 (unregistered)
    Code323 = 323,
    /// 324 (unregistered)
    Code324 = 324,
    /// 325 (unregistered)
    Code325 = 325,
    /// 326 (unregistered)
    Code326 = 326,
    /// 327 (unregistered)
    Code327 = 327,
    /// 328 (unregistered)
    Code328 = 328,
    /// 329 (unregistered)
    Code329 = 329,
    /// 330 (unregistered)
    Code330 = 330,
    /// 331 (unregistered)
    Code331 = 331,
    /// 332 (unregistered)
    Code332 = 332,
    /// 333 (unregistered)
    Code333 = 333,
    /// 334 (unregistered)
    Code334 = 334,
    /// 335 (unregistered)
    Code335 = 335,
    /// 336 (unregistered)
    Code336 = 336,
    /// 337 (unregistered)
    Code337 = 337,
    /// 338 (unregistered)
    Code338 = 338,
    /// 339 (unregistered)
    Code339 = 339,
    /// 340 (unregistered)
    Code340 = 340,
    /// 341 (unregistered)
    Code341 = 341,
    /// 342 (unregistered)
    Code342 = 342,
    /// 343 (unregistered)
    Code343 = 343,
    /// 344 (unregistered)
    Code344 = 344,
    /// 345 (unregistered)
    Code345 = 345,
    /// 346 (unregistered)
    Code346 = 346,
    /// 347 (unregistered)
    Code347 = 347,
    /// 348 (unregistered)
    Code348 = 348,
    /// 349 (unregistered)
    Code349 = 349,
    /// 350 (unregistered)
    Code350 = 350,
    /// 351 (unregistered)
    Code351 = 351,
    /// 352 (unregistered)
    Code352 = 352,
    /// 353 (unregistered)
    Code353 = 353,
    /// 354 (unregistered)
    Code354 = 354,
    /// 355 (unregistered)
    Code355 = 355,
    /// 356 (unregistered)
    Code356 = 356,
    /// 357 (unregistered)
    Code357 = 357,
    /// 358 (unregistered)
    Code358 = 358,
    /// 359 (unregistered)
    Code359 = 359,
    /// 360 (unregistered)
    Code360 = 360,
    /// 361 (unregistered)
    Code361 = 361,
    /// 362 (unregistered)
    Code362 = 362,
    /// 363 (unregistered)
    Code363 = 363,
    /// 364 (unregistered)
    Code364 = 364,
    /// 365 (unregistered)
    Code365 = 365,
    /// 366 (unregistered)
    Code366 = 366,
    /// 367 (unregistered)
    Code367 = 367,
    /// 368 (unregistered)
    Code368 = 368,
    /// 369 (unregistered)
    Code369 = 369,
    /// 370 (unregistered)
    Code370 = 370,
    /// 371 (unregistered)
    Code371 = 371,
    /// 372 (unregistered)
    Code372 = 372,
    /// 373 (unregistered)
    Code373 = 373,
    /// 374 (unregistered)
    Code374 = 374,
    /// 375 (unregistered)
    Code375 = 375,
    /// 376 (unregistered)
    Code376 = 376,
    /// 377 (unregistered)
    Code377 = 377,
    /// 378 (unregistered)
    Code378 = 378,
    /// 379 (unregistered)
    Code379 = 379,
    /// 380 (unregistered)
    Code380 = 380,
    /// 381 (unregistered)
    Code381 = 381,
    /// 382 (unregistered)
    Code382 = 382,
    /// 383 (unregistered)
    Code383 = 383,
    /// 384 (unregistered)
    Code384 = 384,
    /// 385 (unregistered)
    Code385 = 385,
    /// 386 (unregistered)
    Code386 = 386,
    /// 387 (unregistered)
    Code387 = 387,
    /// 388 (unregistered)
    Code388 = 388,
    /// 389 (unregistered)
    Code389 = 389,
    /// 390 (unregistered)
    Code390 = 390,
    /// 391 (unregistered)
    Code391 = 391,
    /// 392 (unregistered)
    Code392 = 392,
    /// 393 (unregistered)
    Code393 = 393,
    /// 394 (unregistered)
    Code394 = 394,
    /// 395 (unregistered)
    Code395 = 395,
    /// 396 (unregistered)
    Code396 = 396,
    /// 397 (unregistered)
    Code397 = 397,
    /// 398 (unregistered)
    Code398 = 398,
    /// 399 (unregistered)
    Code399 = 399,

    /// 400 Bad Request
    /// [[RFC7231, Section 6.5.1](https://tools.ietf.org/html/rfc7231#section-6.5.1)]
    BadRequest = 400,
    /// 401 Unauthorized
    /// [[RFC7235, Section 3.1](https://tools.ietf.org/html/rfc7235#section-3.1)]
    Unauthorized = 401,
    /// 402 Payment Required
    /// [[RFC7231, Section 6.5.2](https://tools.ietf.org/html/rfc7231#section-6.5.2)]
    PaymentRequired = 402,
    /// 403 Forbidden
    /// [[RFC7231, Section 6.5.3](https://tools.ietf.org/html/rfc7231#section-6.5.3)]
    Forbidden = 403,
    /// 404 Not Found
    /// [[RFC7231, Section 6.5.4](https://tools.ietf.org/html/rfc7231#section-6.5.4)]
    NotFound = 404,
    /// 405 Method Not Allowed
    /// [[RFC7231, Section 6.5.5](https://tools.ietf.org/html/rfc7231#section-6.5.5)]
    MethodNotAllowed = 405,
    /// 406 Not Acceptable
    /// [[RFC7231, Section 6.5.6](https://tools.ietf.org/html/rfc7231#section-6.5.6)]
    NotAcceptable = 406,
    /// 407 Proxy Authentication Required
    /// [[RFC7235, Section 3.2](https://tools.ietf.org/html/rfc7235#section-3.2)]
    ProxyAuthenticationRequired = 407,
    /// 408 Request Timeout
    /// [[RFC7231, Section 6.5.7](https://tools.ietf.org/html/rfc7231#section-6.5.7)]
    RequestTimeout = 408,
    /// 409 Conflict
    /// [[RFC7231, Section 6.5.8](https://tools.ietf.org/html/rfc7231#section-6.5.8)]
    Conflict = 409,
    /// 410 Gone
    /// [[RFC7231, Section 6.5.9](https://tools.ietf.org/html/rfc7231#section-6.5.9)]
    Gone = 410,
    /// 411 Length Required
    /// [[RFC7231, Section 6.5.10](https://tools.ietf.org/html/rfc7231#section-6.5.10)]
    LengthRequired = 411,
    /// 412 Precondition Failed
    /// [[RFC7232, Section 4.2](https://tools.ietf.org/html/rfc7232#section-4.2)]
    PreconditionFailed = 412,
    /// 413 Payload Too Large
    /// [[RFC7231, Section 6.5.11](https://tools.ietf.org/html/rfc7231#section-6.5.11)]
    PayloadTooLarge = 413,
    /// 414 URI Too Long
    /// [[RFC7231, Section 6.5.12](https://tools.ietf.org/html/rfc7231#section-6.5.12)]
    UriTooLong = 414,
    /// 415 Unsupported Media Type
    /// [[RFC7231, Section 6.5.13](https://tools.ietf.org/html/rfc7231#section-6.5.13)]
    UnsupportedMediaType = 415,
    /// 416 Range Not Satisfiable
    /// [[RFC7233, Section 4.4](https://tools.ietf.org/html/rfc7233#section-4.4)]
    RangeNotSatisfiable = 416,
    /// 417 Expectation Failed
    /// [[RFC7231, Section 6.5.14](https://tools.ietf.org/html/rfc7231#section-6.5.14)]
    ExpectationFailed = 417,
    /// 418 I'm a teapot
    /// [curiously, not registered by IANA, but [RFC2324](https://tools.ietf.org/html/rfc2324)]
    ImATeapot = 418,
    /// 419 (unregistered)
    Code419 = 419,
    /// 420 (unregistered)
    Code420 = 420,
    /// 421 Misdirected Request
    /// [[RFC7540, Section 9.1.2](https://tools.ietf.org/html/rfc7540#section-9.1.2)]
    MisdirectedRequest = 421,
    /// 422 Unprocessable Entity
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    UnprocessableEntity = 422,
    /// 423 Locked
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    Locked = 423,
    /// 424 Failed Dependency
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    FailedDependency = 424,
    /// 425 (unregistered)
    Code425 = 425,
    /// 426 Upgrade Required
    /// [[RFC7231, Section 6.5.15](https://tools.ietf.org/html/rfc7231#section-6.5.15)]
    UpgradeRequired = 426,
    /// 427 (unregistered)
    Code427 = 427,
    /// 428 Precondition Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    PreconditionRequired = 428,
    /// 429 Too Many Requests
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    TooManyRequests = 429,
    /// 430 (unregistered)
    Code430 = 430,
    /// 431 Request Header Fields Too Large
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    RequestHeaderFieldsTooLarge = 431,
    /// 432 (unregistered)
    Code432 = 432,
    /// 433 (unregistered)
    Code433 = 433,
    /// 434 (unregistered)
    Code434 = 434,
    /// 435 (unregistered)
    Code435 = 435,
    /// 436 (unregistered)
    Code436 = 436,
    /// 437 (unregistered)
    Code437 = 437,
    /// 438 (unregistered)
    Code438 = 438,
    /// 439 (unregistered)
    Code439 = 439,
    /// 440 (unregistered)
    Code440 = 440,
    /// 441 (unregistered)
    Code441 = 441,
    /// 442 (unregistered)
    Code442 = 442,
    /// 443 (unregistered)
    Code443 = 443,
    /// 444 (unregistered)
    Code444 = 444,
    /// 445 (unregistered)
    Code445 = 445,
    /// 446 (unregistered)
    Code446 = 446,
    /// 447 (unregistered)
    Code447 = 447,
    /// 448 (unregistered)
    Code448 = 448,
    /// 449 (unregistered)
    Code449 = 449,
    /// 450 (unregistered)
    Code450 = 450,
    // Wanted 451 Unavailable For Legal Reasons? It expired and wasn’t followed up:
    // https://tools.ietf.org/html/draft-tbray-http-legally-restricted-status-04
    /// 451 (unregistered)
    Code451 = 451,
    /// 452 (unregistered)
    Code452 = 452,
    /// 453 (unregistered)
    Code453 = 453,
    /// 454 (unregistered)
    Code454 = 454,
    /// 455 (unregistered)
    Code455 = 455,
    /// 456 (unregistered)
    Code456 = 456,
    /// 457 (unregistered)
    Code457 = 457,
    /// 458 (unregistered)
    Code458 = 458,
    /// 459 (unregistered)
    Code459 = 459,
    /// 460 (unregistered)
    Code460 = 460,
    /// 461 (unregistered)
    Code461 = 461,
    /// 462 (unregistered)
    Code462 = 462,
    /// 463 (unregistered)
    Code463 = 463,
    /// 464 (unregistered)
    Code464 = 464,
    /// 465 (unregistered)
    Code465 = 465,
    /// 466 (unregistered)
    Code466 = 466,
    /// 467 (unregistered)
    Code467 = 467,
    /// 468 (unregistered)
    Code468 = 468,
    /// 469 (unregistered)
    Code469 = 469,
    /// 470 (unregistered)
    Code470 = 470,
    /// 471 (unregistered)
    Code471 = 471,
    /// 472 (unregistered)
    Code472 = 472,
    /// 473 (unregistered)
    Code473 = 473,
    /// 474 (unregistered)
    Code474 = 474,
    /// 475 (unregistered)
    Code475 = 475,
    /// 476 (unregistered)
    Code476 = 476,
    /// 477 (unregistered)
    Code477 = 477,
    /// 478 (unregistered)
    Code478 = 478,
    /// 479 (unregistered)
    Code479 = 479,
    /// 480 (unregistered)
    Code480 = 480,
    /// 481 (unregistered)
    Code481 = 481,
    /// 482 (unregistered)
    Code482 = 482,
    /// 483 (unregistered)
    Code483 = 483,
    /// 484 (unregistered)
    Code484 = 484,
    /// 485 (unregistered)
    Code485 = 485,
    /// 486 (unregistered)
    Code486 = 486,
    /// 487 (unregistered)
    Code487 = 487,
    /// 488 (unregistered)
    Code488 = 488,
    /// 489 (unregistered)
    Code489 = 489,
    /// 490 (unregistered)
    Code490 = 490,
    /// 491 (unregistered)
    Code491 = 491,
    /// 492 (unregistered)
    Code492 = 492,
    /// 493 (unregistered)
    Code493 = 493,
    /// 494 (unregistered)
    Code494 = 494,
    /// 495 (unregistered)
    Code495 = 495,
    /// 496 (unregistered)
    Code496 = 496,
    /// 497 (unregistered)
    Code497 = 497,
    /// 498 (unregistered)
    Code498 = 498,
    /// 499 (unregistered)
    Code499 = 499,

    /// 500 Internal Server Error
    /// [[RFC7231, Section 6.6.1](https://tools.ietf.org/html/rfc7231#section-6.6.1)]
    InternalServerError = 500,
    /// 501 Not Implemented
    /// [[RFC7231, Section 6.6.2](https://tools.ietf.org/html/rfc7231#section-6.6.2)]
    NotImplemented = 501,
    /// 502 Bad Gateway
    /// [[RFC7231, Section 6.6.3](https://tools.ietf.org/html/rfc7231#section-6.6.3)]
    BadGateway = 502,
    /// 503 Service Unavailable
    /// [[RFC7231, Section 6.6.4](https://tools.ietf.org/html/rfc7231#section-6.6.4)]
    ServiceUnavailable = 503,
    /// 504 Gateway Timeout
    /// [[RFC7231, Section 6.6.5](https://tools.ietf.org/html/rfc7231#section-6.6.5)]
    GatewayTimeout = 504,
    /// 505 HTTP Version Not Supported
    /// [[RFC7231, Section 6.6.6](https://tools.ietf.org/html/rfc7231#section-6.6.6)]
    HttpVersionNotSupported = 505,
    /// 506 Variant Also Negotiates
    /// [[RFC2295](https://tools.ietf.org/html/rfc2295)]
    VariantAlsoNegotiates = 506,
    /// 507 Insufficient Storage
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    InsufficientStorage = 507,
    /// 508 Loop Detected
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    LoopDetected = 508,
    /// 509 (unregistered)
    Code509 = 509,
    /// 510 Not Extended
    /// [[RFC2774](https://tools.ietf.org/html/rfc2774)]
    NotExtended = 510,
    /// 511 Network Authentication Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    NetworkAuthenticationRequired = 511,
    /// 512 (unregistered)
    Code512 = 512,
    /// 513 (unregistered)
    Code513 = 513,
    /// 514 (unregistered)
    Code514 = 514,
    /// 515 (unregistered)
    Code515 = 515,
    /// 516 (unregistered)
    Code516 = 516,
    /// 517 (unregistered)
    Code517 = 517,
    /// 518 (unregistered)
    Code518 = 518,
    /// 519 (unregistered)
    Code519 = 519,
    /// 520 (unregistered)
    Code520 = 520,
    /// 521 (unregistered)
    Code521 = 521,
    /// 522 (unregistered)
    Code522 = 522,
    /// 523 (unregistered)
    Code523 = 523,
    /// 524 (unregistered)
    Code524 = 524,
    /// 525 (unregistered)
    Code525 = 525,
    /// 526 (unregistered)
    Code526 = 526,
    /// 527 (unregistered)
    Code527 = 527,
    /// 528 (unregistered)
    Code528 = 528,
    /// 529 (unregistered)
    Code529 = 529,
    /// 530 (unregistered)
    Code530 = 530,
    /// 531 (unregistered)
    Code531 = 531,
    /// 532 (unregistered)
    Code532 = 532,
    /// 533 (unregistered)
    Code533 = 533,
    /// 534 (unregistered)
    Code534 = 534,
    /// 535 (unregistered)
    Code535 = 535,
    /// 536 (unregistered)
    Code536 = 536,
    /// 537 (unregistered)
    Code537 = 537,
    /// 538 (unregistered)
    Code538 = 538,
    /// 539 (unregistered)
    Code539 = 539,
    /// 540 (unregistered)
    Code540 = 540,
    /// 541 (unregistered)
    Code541 = 541,
    /// 542 (unregistered)
    Code542 = 542,
    /// 543 (unregistered)
    Code543 = 543,
    /// 544 (unregistered)
    Code544 = 544,
    /// 545 (unregistered)
    Code545 = 545,
    /// 546 (unregistered)
    Code546 = 546,
    /// 547 (unregistered)
    Code547 = 547,
    /// 548 (unregistered)
    Code548 = 548,
    /// 549 (unregistered)
    Code549 = 549,
    /// 550 (unregistered)
    Code550 = 550,
    /// 551 (unregistered)
    Code551 = 551,
    /// 552 (unregistered)
    Code552 = 552,
    /// 553 (unregistered)
    Code553 = 553,
    /// 554 (unregistered)
    Code554 = 554,
    /// 555 (unregistered)
    Code555 = 555,
    /// 556 (unregistered)
    Code556 = 556,
    /// 557 (unregistered)
    Code557 = 557,
    /// 558 (unregistered)
    Code558 = 558,
    /// 559 (unregistered)
    Code559 = 559,
    /// 560 (unregistered)
    Code560 = 560,
    /// 561 (unregistered)
    Code561 = 561,
    /// 562 (unregistered)
    Code562 = 562,
    /// 563 (unregistered)
    Code563 = 563,
    /// 564 (unregistered)
    Code564 = 564,
    /// 565 (unregistered)
    Code565 = 565,
    /// 566 (unregistered)
    Code566 = 566,
    /// 567 (unregistered)
    Code567 = 567,
    /// 568 (unregistered)
    Code568 = 568,
    /// 569 (unregistered)
    Code569 = 569,
    /// 570 (unregistered)
    Code570 = 570,
    /// 571 (unregistered)
    Code571 = 571,
    /// 572 (unregistered)
    Code572 = 572,
    /// 573 (unregistered)
    Code573 = 573,
    /// 574 (unregistered)
    Code574 = 574,
    /// 575 (unregistered)
    Code575 = 575,
    /// 576 (unregistered)
    Code576 = 576,
    /// 577 (unregistered)
    Code577 = 577,
    /// 578 (unregistered)
    Code578 = 578,
    /// 579 (unregistered)
    Code579 = 579,
    /// 580 (unregistered)
    Code580 = 580,
    /// 581 (unregistered)
    Code581 = 581,
    /// 582 (unregistered)
    Code582 = 582,
    /// 583 (unregistered)
    Code583 = 583,
    /// 584 (unregistered)
    Code584 = 584,
    /// 585 (unregistered)
    Code585 = 585,
    /// 586 (unregistered)
    Code586 = 586,
    /// 587 (unregistered)
    Code587 = 587,
    /// 588 (unregistered)
    Code588 = 588,
    /// 589 (unregistered)
    Code589 = 589,
    /// 590 (unregistered)
    Code590 = 590,
    /// 591 (unregistered)
    Code591 = 591,
    /// 592 (unregistered)
    Code592 = 592,
    /// 593 (unregistered)
    Code593 = 593,
    /// 594 (unregistered)
    Code594 = 594,
    /// 595 (unregistered)
    Code595 = 595,
    /// 596 (unregistered)
    Code596 = 596,
    /// 597 (unregistered)
    Code597 = 597,
    /// 598 (unregistered)
    Code598 = 598,
    /// 599 (unregistered)
    Code599 = 599,
}

impl StatusCode {

    /// Get the standardised `reason-phrase` for this status code.
    ///
    /// This is mostly here for servers writing responses, but could potentially have application
    /// at other times.
    ///
    /// The reason phrase is defined as being exclusively for human readers. You should avoid
    /// deriving any meaning from it at all costs.
    ///
    /// Bear in mind also that in HTTP/2.0 the reason phrase is abolished from transmission, and so
    /// this canonical reason phrase really is the only reason phrase you’ll find.
    pub fn canonical_reason(&self) -> Option<&'static str> {
        match *self {
            Continue => Some("Continue"),
            SwitchingProtocols => Some("Switching Protocols"),
            Processing => Some("Processing"),
            Code103 => None,
            Code104 => None,
            Code105 => None,
            Code106 => None,
            Code107 => None,
            Code108 => None,
            Code109 => None,
            Code110 => None,
            Code111 => None,
            Code112 => None,
            Code113 => None,
            Code114 => None,
            Code115 => None,
            Code116 => None,
            Code117 => None,
            Code118 => None,
            Code119 => None,
            Code120 => None,
            Code121 => None,
            Code122 => None,
            Code123 => None,
            Code124 => None,
            Code125 => None,
            Code126 => None,
            Code127 => None,
            Code128 => None,
            Code129 => None,
            Code130 => None,
            Code131 => None,
            Code132 => None,
            Code133 => None,
            Code134 => None,
            Code135 => None,
            Code136 => None,
            Code137 => None,
            Code138 => None,
            Code139 => None,
            Code140 => None,
            Code141 => None,
            Code142 => None,
            Code143 => None,
            Code144 => None,
            Code145 => None,
            Code146 => None,
            Code147 => None,
            Code148 => None,
            Code149 => None,
            Code150 => None,
            Code151 => None,
            Code152 => None,
            Code153 => None,
            Code154 => None,
            Code155 => None,
            Code156 => None,
            Code157 => None,
            Code158 => None,
            Code159 => None,
            Code160 => None,
            Code161 => None,
            Code162 => None,
            Code163 => None,
            Code164 => None,
            Code165 => None,
            Code166 => None,
            Code167 => None,
            Code168 => None,
            Code169 => None,
            Code170 => None,
            Code171 => None,
            Code172 => None,
            Code173 => None,
            Code174 => None,
            Code175 => None,
            Code176 => None,
            Code177 => None,
            Code178 => None,
            Code179 => None,
            Code180 => None,
            Code181 => None,
            Code182 => None,
            Code183 => None,
            Code184 => None,
            Code185 => None,
            Code186 => None,
            Code187 => None,
            Code188 => None,
            Code189 => None,
            Code190 => None,
            Code191 => None,
            Code192 => None,
            Code193 => None,
            Code194 => None,
            Code195 => None,
            Code196 => None,
            Code197 => None,
            Code198 => None,
            Code199 => None,

            Ok => Some("OK"),
            Created => Some("Created"),
            Accepted => Some("Accepted"),
            NonAuthoritativeInformation => Some("Non-Authoritative Information"),
            NoContent => Some("No Content"),
            ResetContent => Some("Reset Content"),
            PartialContent => Some("Partial Content"),
            MultiStatus => Some("Multi-Status"),
            AlreadyReported => Some("Already Reported"),
            Code209 => None,
            Code210 => None,
            Code211 => None,
            Code212 => None,
            Code213 => None,
            Code214 => None,
            Code215 => None,
            Code216 => None,
            Code217 => None,
            Code218 => None,
            Code219 => None,
            Code220 => None,
            Code221 => None,
            Code222 => None,
            Code223 => None,
            Code224 => None,
            Code225 => None,
            ImUsed => Some("IM Used"),
            Code227 => None,
            Code228 => None,
            Code229 => None,
            Code230 => None,
            Code231 => None,
            Code232 => None,
            Code233 => None,
            Code234 => None,
            Code235 => None,
            Code236 => None,
            Code237 => None,
            Code238 => None,
            Code239 => None,
            Code240 => None,
            Code241 => None,
            Code242 => None,
            Code243 => None,
            Code244 => None,
            Code245 => None,
            Code246 => None,
            Code247 => None,
            Code248 => None,
            Code249 => None,
            Code250 => None,
            Code251 => None,
            Code252 => None,
            Code253 => None,
            Code254 => None,
            Code255 => None,
            Code256 => None,
            Code257 => None,
            Code258 => None,
            Code259 => None,
            Code260 => None,
            Code261 => None,
            Code262 => None,
            Code263 => None,
            Code264 => None,
            Code265 => None,
            Code266 => None,
            Code267 => None,
            Code268 => None,
            Code269 => None,
            Code270 => None,
            Code271 => None,
            Code272 => None,
            Code273 => None,
            Code274 => None,
            Code275 => None,
            Code276 => None,
            Code277 => None,
            Code278 => None,
            Code279 => None,
            Code280 => None,
            Code281 => None,
            Code282 => None,
            Code283 => None,
            Code284 => None,
            Code285 => None,
            Code286 => None,
            Code287 => None,
            Code288 => None,
            Code289 => None,
            Code290 => None,
            Code291 => None,
            Code292 => None,
            Code293 => None,
            Code294 => None,
            Code295 => None,
            Code296 => None,
            Code297 => None,
            Code298 => None,
            Code299 => None,

            MultipleChoices => Some("Multiple Choices"),
            MovedPermanently => Some("Moved Permanently"),
            Found => Some("Found"),
            SeeOther => Some("See Other"),
            NotModified => Some("Not Modified"),
            UseProxy => Some("Use Proxy"),
            Code306 => None,
            TemporaryRedirect => Some("Temporary Redirect"),
            PermanentRedirect => Some("Permanent Redirect"),
            Code309 => None,
            Code310 => None,
            Code311 => None,
            Code312 => None,
            Code313 => None,
            Code314 => None,
            Code315 => None,
            Code316 => None,
            Code317 => None,
            Code318 => None,
            Code319 => None,
            Code320 => None,
            Code321 => None,
            Code322 => None,
            Code323 => None,
            Code324 => None,
            Code325 => None,
            Code326 => None,
            Code327 => None,
            Code328 => None,
            Code329 => None,
            Code330 => None,
            Code331 => None,
            Code332 => None,
            Code333 => None,
            Code334 => None,
            Code335 => None,
            Code336 => None,
            Code337 => None,
            Code338 => None,
            Code339 => None,
            Code340 => None,
            Code341 => None,
            Code342 => None,
            Code343 => None,
            Code344 => None,
            Code345 => None,
            Code346 => None,
            Code347 => None,
            Code348 => None,
            Code349 => None,
            Code350 => None,
            Code351 => None,
            Code352 => None,
            Code353 => None,
            Code354 => None,
            Code355 => None,
            Code356 => None,
            Code357 => None,
            Code358 => None,
            Code359 => None,
            Code360 => None,
            Code361 => None,
            Code362 => None,
            Code363 => None,
            Code364 => None,
            Code365 => None,
            Code366 => None,
            Code367 => None,
            Code368 => None,
            Code369 => None,
            Code370 => None,
            Code371 => None,
            Code372 => None,
            Code373 => None,
            Code374 => None,
            Code375 => None,
            Code376 => None,
            Code377 => None,
            Code378 => None,
            Code379 => None,
            Code380 => None,
            Code381 => None,
            Code382 => None,
            Code383 => None,
            Code384 => None,
            Code385 => None,
            Code386 => None,
            Code387 => None,
            Code388 => None,
            Code389 => None,
            Code390 => None,
            Code391 => None,
            Code392 => None,
            Code393 => None,
            Code394 => None,
            Code395 => None,
            Code396 => None,
            Code397 => None,
            Code398 => None,
            Code399 => None,

            BadRequest => Some("Bad Request"),
            Unauthorized => Some("Unauthorized"),
            PaymentRequired => Some("Payment Required"),
            Forbidden => Some("Forbidden"),
            NotFound => Some("Not Found"),
            MethodNotAllowed => Some("Method Not Allowed"),
            NotAcceptable => Some("Not Acceptable"),
            ProxyAuthenticationRequired => Some("Proxy Authentication Required"),
            RequestTimeout => Some("Request Timeout"),
            Conflict => Some("Conflict"),
            Gone => Some("Gone"),
            LengthRequired => Some("Length Required"),
            PreconditionFailed => Some("Precondition Failed"),
            PayloadTooLarge => Some("Payload Too Large"),
            UriTooLong => Some("URI Too Long"),
            UnsupportedMediaType => Some("Unsupported Media Type"),
            RangeNotSatisfiable => Some("Range Not Satisfiable"),
            ExpectationFailed => Some("Expectation Failed"),
            ImATeapot => Some("I'm a teapot"),
            Code419 => None,
            Code420 => None,
            MisdirectedRequest => Some("Misdirected Request"),
            UnprocessableEntity => Some("Unprocessable Entity"),
            Locked => Some("Locked"),
            FailedDependency => Some("Failed Dependency"),
            Code425 => None,
            UpgradeRequired => Some("Upgrade Required"),
            Code427 => None,
            PreconditionRequired => Some("Precondition Required"),
            TooManyRequests => Some("Too Many Requests"),
            Code430 => None,
            RequestHeaderFieldsTooLarge => Some("Request Header Fields Too Large"),
            Code432 => None,
            Code433 => None,
            Code434 => None,
            Code435 => None,
            Code436 => None,
            Code437 => None,
            Code438 => None,
            Code439 => None,
            Code440 => None,
            Code441 => None,
            Code442 => None,
            Code443 => None,
            Code444 => None,
            Code445 => None,
            Code446 => None,
            Code447 => None,
            Code448 => None,
            Code449 => None,
            Code450 => None,
            Code451 => None,
            Code452 => None,
            Code453 => None,
            Code454 => None,
            Code455 => None,
            Code456 => None,
            Code457 => None,
            Code458 => None,
            Code459 => None,
            Code460 => None,
            Code461 => None,
            Code462 => None,
            Code463 => None,
            Code464 => None,
            Code465 => None,
            Code466 => None,
            Code467 => None,
            Code468 => None,
            Code469 => None,
            Code470 => None,
            Code471 => None,
            Code472 => None,
            Code473 => None,
            Code474 => None,
            Code475 => None,
            Code476 => None,
            Code477 => None,
            Code478 => None,
            Code479 => None,
            Code480 => None,
            Code481 => None,
            Code482 => None,
            Code483 => None,
            Code484 => None,
            Code485 => None,
            Code486 => None,
            Code487 => None,
            Code488 => None,
            Code489 => None,
            Code490 => None,
            Code491 => None,
            Code492 => None,
            Code493 => None,
            Code494 => None,
            Code495 => None,
            Code496 => None,
            Code497 => None,
            Code498 => None,
            Code499 => None,

            InternalServerError => Some("Internal Server Error"),
            NotImplemented => Some("Not Implemented"),
            BadGateway => Some("Bad Gateway"),
            ServiceUnavailable => Some("Service Unavailable"),
            GatewayTimeout => Some("Gateway Timeout"),
            HttpVersionNotSupported => Some("HTTP Version Not Supported"),
            VariantAlsoNegotiates => Some("Variant Also Negotiates"),
            InsufficientStorage => Some("Insufficient Storage"),
            LoopDetected => Some("Loop Detected"),
            Code509 => None,
            NotExtended => Some("Not Extended"),
            NetworkAuthenticationRequired => Some("Network Authentication Required"),
            Code512 => None,
            Code513 => None,
            Code514 => None,
            Code515 => None,
            Code516 => None,
            Code517 => None,
            Code518 => None,
            Code519 => None,
            Code520 => None,
            Code521 => None,
            Code522 => None,
            Code523 => None,
            Code524 => None,
            Code525 => None,
            Code526 => None,
            Code527 => None,
            Code528 => None,
            Code529 => None,
            Code530 => None,
            Code531 => None,
            Code532 => None,
            Code533 => None,
            Code534 => None,
            Code535 => None,
            Code536 => None,
            Code537 => None,
            Code538 => None,
            Code539 => None,
            Code540 => None,
            Code541 => None,
            Code542 => None,
            Code543 => None,
            Code544 => None,
            Code545 => None,
            Code546 => None,
            Code547 => None,
            Code548 => None,
            Code549 => None,
            Code550 => None,
            Code551 => None,
            Code552 => None,
            Code553 => None,
            Code554 => None,
            Code555 => None,
            Code556 => None,
            Code557 => None,
            Code558 => None,
            Code559 => None,
            Code560 => None,
            Code561 => None,
            Code562 => None,
            Code563 => None,
            Code564 => None,
            Code565 => None,
            Code566 => None,
            Code567 => None,
            Code568 => None,
            Code569 => None,
            Code570 => None,
            Code571 => None,
            Code572 => None,
            Code573 => None,
            Code574 => None,
            Code575 => None,
            Code576 => None,
            Code577 => None,
            Code578 => None,
            Code579 => None,
            Code580 => None,
            Code581 => None,
            Code582 => None,
            Code583 => None,
            Code584 => None,
            Code585 => None,
            Code586 => None,
            Code587 => None,
            Code588 => None,
            Code589 => None,
            Code590 => None,
            Code591 => None,
            Code592 => None,
            Code593 => None,
            Code594 => None,
            Code595 => None,
            Code596 => None,
            Code597 => None,
            Code598 => None,
            Code599 => None,
        }
    }

    /// Determine the class of a status code, based on its first digit.
    pub fn class(&self) -> StatusClass {
        let code = *self as u16;  // Range of possible values: 100..599.
        // We could match 100..199 &c., but this way we avoid unreachable!() at the end.
        if code < 200 {
            Informational
        } else if code < 300 {
            Successful
        } else if code < 400 {
            Redirection
        } else if code < 500 {
            ClientError
        } else {
            ServerError
        }
    }
}

impl Copy for StatusCode {}

/// Formats the status code, *including* the canonical reason.
///
/// ```rust
/// # use teepee::status::{ImATeapot, Code123};
/// assert_eq!(format!("{}", ImATeapot),
///            "418 I'm a teapot");
/// assert_eq!(format!("{}", Code123),
///            "123 <unknown status code>");
/// ```
///
/// If you wish to just include the number, cast to a `u16` instead:
///
/// ```rust
/// # use teepee::status::{ImATeapot, Code123};
/// assert_eq!(format!("{}", ImATeapot as u16), "418");
/// assert_eq!(format!("{}", Code123 as u16), "123");
/// ```
impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", *self as u16,
               self.canonical_reason().unwrap_or("<unknown status code>"))
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Continue => "Continue",
            SwitchingProtocols => "SwitchingProtocols",
            Processing => "Processing",
            Code103 => "Code103",
            Code104 => "Code104",
            Code105 => "Code105",
            Code106 => "Code106",
            Code107 => "Code107",
            Code108 => "Code108",
            Code109 => "Code109",
            Code110 => "Code110",
            Code111 => "Code111",
            Code112 => "Code112",
            Code113 => "Code113",
            Code114 => "Code114",
            Code115 => "Code115",
            Code116 => "Code116",
            Code117 => "Code117",
            Code118 => "Code118",
            Code119 => "Code119",
            Code120 => "Code120",
            Code121 => "Code121",
            Code122 => "Code122",
            Code123 => "Code123",
            Code124 => "Code124",
            Code125 => "Code125",
            Code126 => "Code126",
            Code127 => "Code127",
            Code128 => "Code128",
            Code129 => "Code129",
            Code130 => "Code130",
            Code131 => "Code131",
            Code132 => "Code132",
            Code133 => "Code133",
            Code134 => "Code134",
            Code135 => "Code135",
            Code136 => "Code136",
            Code137 => "Code137",
            Code138 => "Code138",
            Code139 => "Code139",
            Code140 => "Code140",
            Code141 => "Code141",
            Code142 => "Code142",
            Code143 => "Code143",
            Code144 => "Code144",
            Code145 => "Code145",
            Code146 => "Code146",
            Code147 => "Code147",
            Code148 => "Code148",
            Code149 => "Code149",
            Code150 => "Code150",
            Code151 => "Code151",
            Code152 => "Code152",
            Code153 => "Code153",
            Code154 => "Code154",
            Code155 => "Code155",
            Code156 => "Code156",
            Code157 => "Code157",
            Code158 => "Code158",
            Code159 => "Code159",
            Code160 => "Code160",
            Code161 => "Code161",
            Code162 => "Code162",
            Code163 => "Code163",
            Code164 => "Code164",
            Code165 => "Code165",
            Code166 => "Code166",
            Code167 => "Code167",
            Code168 => "Code168",
            Code169 => "Code169",
            Code170 => "Code170",
            Code171 => "Code171",
            Code172 => "Code172",
            Code173 => "Code173",
            Code174 => "Code174",
            Code175 => "Code175",
            Code176 => "Code176",
            Code177 => "Code177",
            Code178 => "Code178",
            Code179 => "Code179",
            Code180 => "Code180",
            Code181 => "Code181",
            Code182 => "Code182",
            Code183 => "Code183",
            Code184 => "Code184",
            Code185 => "Code185",
            Code186 => "Code186",
            Code187 => "Code187",
            Code188 => "Code188",
            Code189 => "Code189",
            Code190 => "Code190",
            Code191 => "Code191",
            Code192 => "Code192",
            Code193 => "Code193",
            Code194 => "Code194",
            Code195 => "Code195",
            Code196 => "Code196",
            Code197 => "Code197",
            Code198 => "Code198",
            Code199 => "Code199",

            Ok => "Ok",
            Created => "Created",
            Accepted => "Accepted",
            NonAuthoritativeInformation => "NonAuthoritativeInformation",
            NoContent => "NoContent",
            ResetContent => "ResetContent",
            PartialContent => "PartialContent",
            MultiStatus => "MultiStatus",
            AlreadyReported => "AlreadyReported",
            Code209 => "Code209",
            Code210 => "Code210",
            Code211 => "Code211",
            Code212 => "Code212",
            Code213 => "Code213",
            Code214 => "Code214",
            Code215 => "Code215",
            Code216 => "Code216",
            Code217 => "Code217",
            Code218 => "Code218",
            Code219 => "Code219",
            Code220 => "Code220",
            Code221 => "Code221",
            Code222 => "Code222",
            Code223 => "Code223",
            Code224 => "Code224",
            Code225 => "Code225",
            ImUsed => "ImUsed",
            Code227 => "Code227",
            Code228 => "Code228",
            Code229 => "Code229",
            Code230 => "Code230",
            Code231 => "Code231",
            Code232 => "Code232",
            Code233 => "Code233",
            Code234 => "Code234",
            Code235 => "Code235",
            Code236 => "Code236",
            Code237 => "Code237",
            Code238 => "Code238",
            Code239 => "Code239",
            Code240 => "Code240",
            Code241 => "Code241",
            Code242 => "Code242",
            Code243 => "Code243",
            Code244 => "Code244",
            Code245 => "Code245",
            Code246 => "Code246",
            Code247 => "Code247",
            Code248 => "Code248",
            Code249 => "Code249",
            Code250 => "Code250",
            Code251 => "Code251",
            Code252 => "Code252",
            Code253 => "Code253",
            Code254 => "Code254",
            Code255 => "Code255",
            Code256 => "Code256",
            Code257 => "Code257",
            Code258 => "Code258",
            Code259 => "Code259",
            Code260 => "Code260",
            Code261 => "Code261",
            Code262 => "Code262",
            Code263 => "Code263",
            Code264 => "Code264",
            Code265 => "Code265",
            Code266 => "Code266",
            Code267 => "Code267",
            Code268 => "Code268",
            Code269 => "Code269",
            Code270 => "Code270",
            Code271 => "Code271",
            Code272 => "Code272",
            Code273 => "Code273",
            Code274 => "Code274",
            Code275 => "Code275",
            Code276 => "Code276",
            Code277 => "Code277",
            Code278 => "Code278",
            Code279 => "Code279",
            Code280 => "Code280",
            Code281 => "Code281",
            Code282 => "Code282",
            Code283 => "Code283",
            Code284 => "Code284",
            Code285 => "Code285",
            Code286 => "Code286",
            Code287 => "Code287",
            Code288 => "Code288",
            Code289 => "Code289",
            Code290 => "Code290",
            Code291 => "Code291",
            Code292 => "Code292",
            Code293 => "Code293",
            Code294 => "Code294",
            Code295 => "Code295",
            Code296 => "Code296",
            Code297 => "Code297",
            Code298 => "Code298",
            Code299 => "Code299",

            MultipleChoices => "MultipleChoices",
            MovedPermanently => "MovedPermanently",
            Found => "Found",
            SeeOther => "SeeOther",
            NotModified => "NotModified",
            UseProxy => "UseProxy",
            Code306 => "Code306",
            TemporaryRedirect => "TemporaryRedirect",
            PermanentRedirect => "PermanentRedirect",
            Code309 => "Code309",
            Code310 => "Code310",
            Code311 => "Code311",
            Code312 => "Code312",
            Code313 => "Code313",
            Code314 => "Code314",
            Code315 => "Code315",
            Code316 => "Code316",
            Code317 => "Code317",
            Code318 => "Code318",
            Code319 => "Code319",
            Code320 => "Code320",
            Code321 => "Code321",
            Code322 => "Code322",
            Code323 => "Code323",
            Code324 => "Code324",
            Code325 => "Code325",
            Code326 => "Code326",
            Code327 => "Code327",
            Code328 => "Code328",
            Code329 => "Code329",
            Code330 => "Code330",
            Code331 => "Code331",
            Code332 => "Code332",
            Code333 => "Code333",
            Code334 => "Code334",
            Code335 => "Code335",
            Code336 => "Code336",
            Code337 => "Code337",
            Code338 => "Code338",
            Code339 => "Code339",
            Code340 => "Code340",
            Code341 => "Code341",
            Code342 => "Code342",
            Code343 => "Code343",
            Code344 => "Code344",
            Code345 => "Code345",
            Code346 => "Code346",
            Code347 => "Code347",
            Code348 => "Code348",
            Code349 => "Code349",
            Code350 => "Code350",
            Code351 => "Code351",
            Code352 => "Code352",
            Code353 => "Code353",
            Code354 => "Code354",
            Code355 => "Code355",
            Code356 => "Code356",
            Code357 => "Code357",
            Code358 => "Code358",
            Code359 => "Code359",
            Code360 => "Code360",
            Code361 => "Code361",
            Code362 => "Code362",
            Code363 => "Code363",
            Code364 => "Code364",
            Code365 => "Code365",
            Code366 => "Code366",
            Code367 => "Code367",
            Code368 => "Code368",
            Code369 => "Code369",
            Code370 => "Code370",
            Code371 => "Code371",
            Code372 => "Code372",
            Code373 => "Code373",
            Code374 => "Code374",
            Code375 => "Code375",
            Code376 => "Code376",
            Code377 => "Code377",
            Code378 => "Code378",
            Code379 => "Code379",
            Code380 => "Code380",
            Code381 => "Code381",
            Code382 => "Code382",
            Code383 => "Code383",
            Code384 => "Code384",
            Code385 => "Code385",
            Code386 => "Code386",
            Code387 => "Code387",
            Code388 => "Code388",
            Code389 => "Code389",
            Code390 => "Code390",
            Code391 => "Code391",
            Code392 => "Code392",
            Code393 => "Code393",
            Code394 => "Code394",
            Code395 => "Code395",
            Code396 => "Code396",
            Code397 => "Code397",
            Code398 => "Code398",
            Code399 => "Code399",

            BadRequest => "BadRequest",
            Unauthorized => "Unauthorized",
            PaymentRequired => "PaymentRequired",
            Forbidden => "Forbidden",
            NotFound => "NotFound",
            MethodNotAllowed => "MethodNotAllowed",
            NotAcceptable => "NotAcceptable",
            ProxyAuthenticationRequired => "ProxyAuthenticationRequired",
            RequestTimeout => "RequestTimeout",
            Conflict => "Conflict",
            Gone => "Gone",
            LengthRequired => "LengthRequired",
            PreconditionFailed => "PreconditionFailed",
            PayloadTooLarge => "PayloadTooLarge",
            UriTooLong => "UriTooLong",
            UnsupportedMediaType => "UnsupportedMediaType",
            RangeNotSatisfiable => "RangeNotSatisfiable",
            ExpectationFailed => "ExpectationFailed",
            ImATeapot => "ImATeapot",
            Code419 => "Code419",
            Code420 => "Code420",
            MisdirectedRequest => "MisdirectedRequest",
            UnprocessableEntity => "UnprocessableEntity",
            Locked => "Locked",
            FailedDependency => "FailedDependency",
            Code425 => "Code425",
            UpgradeRequired => "UpgradeRequired",
            Code427 => "Code427",
            PreconditionRequired => "PreconditionRequired",
            TooManyRequests => "TooManyRequests",
            Code430 => "Code430",
            RequestHeaderFieldsTooLarge => "RequestHeaderFieldsTooLarge",
            Code432 => "Code432",
            Code433 => "Code433",
            Code434 => "Code434",
            Code435 => "Code435",
            Code436 => "Code436",
            Code437 => "Code437",
            Code438 => "Code438",
            Code439 => "Code439",
            Code440 => "Code440",
            Code441 => "Code441",
            Code442 => "Code442",
            Code443 => "Code443",
            Code444 => "Code444",
            Code445 => "Code445",
            Code446 => "Code446",
            Code447 => "Code447",
            Code448 => "Code448",
            Code449 => "Code449",
            Code450 => "Code450",
    // Wanted 451 Unavailable For Legal Reasons? It expired and wasn’t followed up:
    // https://tools.ietf.org/html/draft-tbray-http-legally-restricted-status-04
            Code451 => "Code451",
            Code452 => "Code452",
            Code453 => "Code453",
            Code454 => "Code454",
            Code455 => "Code455",
            Code456 => "Code456",
            Code457 => "Code457",
            Code458 => "Code458",
            Code459 => "Code459",
            Code460 => "Code460",
            Code461 => "Code461",
            Code462 => "Code462",
            Code463 => "Code463",
            Code464 => "Code464",
            Code465 => "Code465",
            Code466 => "Code466",
            Code467 => "Code467",
            Code468 => "Code468",
            Code469 => "Code469",
            Code470 => "Code470",
            Code471 => "Code471",
            Code472 => "Code472",
            Code473 => "Code473",
            Code474 => "Code474",
            Code475 => "Code475",
            Code476 => "Code476",
            Code477 => "Code477",
            Code478 => "Code478",
            Code479 => "Code479",
            Code480 => "Code480",
            Code481 => "Code481",
            Code482 => "Code482",
            Code483 => "Code483",
            Code484 => "Code484",
            Code485 => "Code485",
            Code486 => "Code486",
            Code487 => "Code487",
            Code488 => "Code488",
            Code489 => "Code489",
            Code490 => "Code490",
            Code491 => "Code491",
            Code492 => "Code492",
            Code493 => "Code493",
            Code494 => "Code494",
            Code495 => "Code495",
            Code496 => "Code496",
            Code497 => "Code497",
            Code498 => "Code498",
            Code499 => "Code499",

            InternalServerError => "InternalServerError",
            NotImplemented => "NotImplemented",
            BadGateway => "BadGateway",
            ServiceUnavailable => "ServiceUnavailable",
            GatewayTimeout => "GatewayTimeout",
            HttpVersionNotSupported => "HttpVersionNotSupported",
            VariantAlsoNegotiates => "VariantAlsoNegotiates",
            InsufficientStorage => "InsufficientStorage",
            LoopDetected => "LoopDetected",
            Code509 => "Code509",
            NotExtended => "NotExtended",
            NetworkAuthenticationRequired => "NetworkAuthenticationRequired",
            Code512 => "Code512",
            Code513 => "Code513",
            Code514 => "Code514",
            Code515 => "Code515",
            Code516 => "Code516",
            Code517 => "Code517",
            Code518 => "Code518",
            Code519 => "Code519",
            Code520 => "Code520",
            Code521 => "Code521",
            Code522 => "Code522",
            Code523 => "Code523",
            Code524 => "Code524",
            Code525 => "Code525",
            Code526 => "Code526",
            Code527 => "Code527",
            Code528 => "Code528",
            Code529 => "Code529",
            Code530 => "Code530",
            Code531 => "Code531",
            Code532 => "Code532",
            Code533 => "Code533",
            Code534 => "Code534",
            Code535 => "Code535",
            Code536 => "Code536",
            Code537 => "Code537",
            Code538 => "Code538",
            Code539 => "Code539",
            Code540 => "Code540",
            Code541 => "Code541",
            Code542 => "Code542",
            Code543 => "Code543",
            Code544 => "Code544",
            Code545 => "Code545",
            Code546 => "Code546",
            Code547 => "Code547",
            Code548 => "Code548",
            Code549 => "Code549",
            Code550 => "Code550",
            Code551 => "Code551",
            Code552 => "Code552",
            Code553 => "Code553",
            Code554 => "Code554",
            Code555 => "Code555",
            Code556 => "Code556",
            Code557 => "Code557",
            Code558 => "Code558",
            Code559 => "Code559",
            Code560 => "Code560",
            Code561 => "Code561",
            Code562 => "Code562",
            Code563 => "Code563",
            Code564 => "Code564",
            Code565 => "Code565",
            Code566 => "Code566",
            Code567 => "Code567",
            Code568 => "Code568",
            Code569 => "Code569",
            Code570 => "Code570",
            Code571 => "Code571",
            Code572 => "Code572",
            Code573 => "Code573",
            Code574 => "Code574",
            Code575 => "Code575",
            Code576 => "Code576",
            Code577 => "Code577",
            Code578 => "Code578",
            Code579 => "Code579",
            Code580 => "Code580",
            Code581 => "Code581",
            Code582 => "Code582",
            Code583 => "Code583",
            Code584 => "Code584",
            Code585 => "Code585",
            Code586 => "Code586",
            Code587 => "Code587",
            Code588 => "Code588",
            Code589 => "Code589",
            Code590 => "Code590",
            Code591 => "Code591",
            Code592 => "Code592",
            Code593 => "Code593",
            Code594 => "Code594",
            Code595 => "Code595",
            Code596 => "Code596",
            Code597 => "Code597",
            Code598 => "Code598",
            Code599 => "Code599",
        })
    }
}

// Specified manually because the codegen for derived is slow (at the time of writing on the
// machine of writing, 1.2 seconds) and verbose (though the optimiser cuts it down to size).
impl PartialEq for StatusCode {
    #[inline]
    fn eq(&self, other: &StatusCode) -> bool {
        *self as u16 == *other as u16
    }
}

impl Eq for StatusCode {}

// Ditto (though #[derive(Clone)] only takes about 0.4 seconds).
impl Clone for StatusCode {
    #[inline]
    fn clone(&self) -> StatusCode {
        *self
    }
}

// Of the other common derivable traits, I didn’t measure them, but I guess they would be slow too.

impl StatusCode {
    /// Convert a `u16` to a `StatusCode` if it is in the legal range (100–599).
    pub fn from_u16(n: u16) -> Result<StatusCode, ()> {
        if n >= 100 && n <= 599 {
            Result::Ok(unsafe { mem::transmute(n) })
        } else {
            Err(())
        }
    }
}

impl PartialOrd for StatusCode {
    #[inline]
    fn partial_cmp(&self, other: &StatusCode) -> Option<Ordering> {
        (*self as u16).partial_cmp(&(*other as u16))
    }
}

impl Ord for StatusCode {
    #[inline]
    fn cmp(&self, other: &StatusCode) -> Ordering {
        if *self < *other {
            Ordering::Less
        } else if *self > *other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

/// The class of an HTTP `status-code`.
///
/// [RFC 7231, section 6 (Response Status Codes)](https://tools.ietf.org/html/rfc7231#section-6):
///
/// > The first digit of the status-code defines the class of response.
/// > The last two digits do not have any categorization role.
///
/// And:
///
/// > HTTP status codes are extensible.  HTTP clients are not required to
/// > understand the meaning of all registered status codes, though such
/// > understanding is obviously desirable.  However, a client MUST
/// > understand the class of any status code, as indicated by the first
/// > digit, and treat an unrecognized status code as being equivalent to
/// > the x00 status code of that class, with the exception that a
/// > recipient MUST NOT cache a response with an unrecognized status code.
/// >
/// > For example, if an unrecognized status code of 471 is received by a
/// > client, the client can assume that there was something wrong with its
/// > request and treat the response as if it had received a 400 (Bad
/// > Request) status code.  The response message will usually contain a
/// > representation that explains the status.
///
/// This can be used in cases where a status code’s meaning is unknown, also,
/// to get the appropriate *category* of status.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum StatusClass {
    /// 1xx (Informational): The request was received, continuing process
    Informational = 100,

    /// 2xx (Successful): The request was successfully received, understood, and accepted
    Successful = 200,

    /// 3xx (Redirection): Further action needs to be taken in order to complete the request
    Redirection = 300,

    /// 4xx (Client Error): The request contains bad syntax or cannot be fulfilled
    ClientError = 400,

    /// 5xx (Server Error): The server failed to fulfill an apparently valid request
    ServerError = 500,
}

impl StatusClass {
    /// Get the default status code for the class.
    ///
    /// This produces the x00 status code; thus, for `ClientError` (4xx), for example, this will
    /// produce `BadRequest` (400):
    ///
    /// ```rust
    /// # use teepee::status::{ClientError, BadRequest};
    /// assert_eq!(ClientError.default_code(), BadRequest);
    /// ```
    ///
    /// The use for this is outlined in [RFC 7231, section 6 (Response Status
    /// Codes)](https://tools.ietf.org/html/rfc7231#section-6):
    ///
    /// > HTTP status codes are extensible.  HTTP clients are not required to
    /// > understand the meaning of all registered status codes, though such
    /// > understanding is obviously desirable.  However, a client MUST
    /// > understand the class of any status code, as indicated by the first
    /// > digit, and treat an unrecognized status code as being equivalent to
    /// > the x00 status code of that class, with the exception that a
    /// > recipient MUST NOT cache a response with an unrecognized status code.
    /// >
    /// > For example, if an unrecognized status code of 471 is received by a
    /// > client, the client can assume that there was something wrong with its
    /// > request and treat the response as if it had received a 400 (Bad
    /// > Request) status code.  The response message will usually contain a
    /// > representation that explains the status.
    ///
    /// This is demonstrated thusly:
    ///
    /// ```rust
    /// # use teepee::status::{Code471, BadRequest};
    /// // Suppose we have received this status code.
    /// let status = Code471;
    ///
    /// // Uh oh! Don’t know what to do with it.
    /// // Let’s fall back to the default:
    /// let status = status.class().default_code();
    ///
    /// // And look! That is 400 Bad Request.
    /// assert_eq!(status, BadRequest);
    /// // So now let’s treat it as that.
    /// ```
    #[inline]
    pub fn default_code(&self) -> StatusCode {
        unsafe { mem::transmute(*self) }
    }
}
