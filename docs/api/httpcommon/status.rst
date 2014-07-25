:mod:`status` --- HTTP status codes
===================================

.. module:: status

.. admonition:: Not for direct usage in :crate:`httpcommon`

   Avoid using this module from :crate:`httpcommon`; it is reexported in the crates where it is relevant:
   
   - In :crate:`httpc`, :mod:`httpc::status`;
   - In :crate:`httpd`, :mod:`httpd::status`.

Module contents:

- :enum:`StatusCode` --- HTTP status codes
- :enum:`StatusClass` --- the class of an HTTP Status-Code

.. _httpcommon-status-statuscode:

:enum:`StatusCode` --- HTTP status codes
----------------------------------------

.. enum:: StatusCode

   (Five hundred variants, omitted here for easier comprehension.)

An HTTP status code (``Status-Code`` in :rfc:`2616`).

This enum is absolutely exhaustive, covering all 500 possible values (100--599).

For HTTP/2.0, statuses belonging to the 1xx Informational class are invalid.

As this is a C-style enum with each variant having a corresponding value, you
may use the likes of ``Continue as u16`` to retreive the value ``100u16``.
Normally, though, you should not need to do any such thing; just use the status
code as a ``StatusCode``.

If you encounter a status code that you do not know how to deal with, you
should treat it as the ``x00`` status code---e.g. for code 123, treat it as 100
(Continue). This can be achieved with ``self.class().default_code()``::

   >>> use httpcommon::status::{Code123, Continue};  // #hide
   >>> Code123.class().default_code()
   Continue

Here is the correspondence of variants inside this enum to their Status-Codes
and canonical Reason-Phrases. Some of these come from :rfc:`2616`, others come
from other places.

.. todo:: add a reference for each one

========================================== ==== ===============================
Variant                                    Code Canonical reason phrase        
========================================== ==== ===============================
.. variant:: Continue                      100  Continue                       
.. variant:: SwitchingProtocols            101  Switching Protocols            
.. variant:: Processing                    102  Processing                     
.. variant:: Ok                            200  OK                             
.. variant:: Created                       201  Created                        
.. variant:: Accepted                      202  Accepted                       
.. variant:: NonAuthoritativeInformation   203  Non-Authoritative Information  
.. variant:: NoContent                     204  No Content                     
.. variant:: ResetContent                  205  Reset Content                  
.. variant:: PartialContent                206  Partial Content                
.. variant:: MultiStatus                   207  Multi-Status                   
.. variant:: AlreadyReported               208  Already Reported               
.. variant:: ImUsed                        226  IM Used                        
.. variant:: MultipleChoices               300  Multiple Choices               
.. variant:: MovedPermanently              301  Moved Permanently              
.. variant:: Found                         302  Found                          
.. variant:: SeeOther                      303  See Other                      
.. variant:: NotModified                   304  Not Modified                   
.. variant:: UseProxy                      305  Use Proxy                      
.. variant:: SwitchProxy                   306  Switch Proxy                   
.. variant:: TemporaryRedirect             307  Temporary Redirect             
.. variant:: PermanentRedirect             308  Permanent Redirect             
.. variant:: BadRequest                    400  Bad Request                    
.. variant:: Unauthorized                  401  Unauthorized                   
.. variant:: PaymentRequired               402  Payment Required               
.. variant:: Forbidden                     403  Forbidden                      
.. variant:: NotFound                      404  Not Found                      
.. variant:: MethodNotAllowed              405  Method Not Allowed             
.. variant:: NotAcceptable                 406  Not Acceptable                 
.. variant:: ProxyAuthenticationRequired   407  Proxy Authentication Required  
.. variant:: RequestTimeout                408  Request Timeout                
.. variant:: Conflict                      409  Conflict                       
.. variant:: Gone                          410  Gone                           
.. variant:: LengthRequired                411  Length Required                
.. variant:: PreconditionFailed            412  Precondition Failed            
.. variant:: RequestEntityTooLarge         413  Request Entity Too Large       
.. variant:: RequestUriTooLong             414  Request-URI Too Long           
.. variant:: UnsupportedMediaType          415  Unsupported Media Type         
.. variant:: RequestedRangeNotSatisfiable  416  Requested Range Not Satisfiable
.. variant:: ExpectationFailed             417  Expectation Failed             
.. variant:: ImATeapot                     418  I'm a teapot                   
.. variant:: AuthenticationTimeout         419  Authentication Timeout         
.. variant:: UnprocessableEntity           422  Unprocessable Entity           
.. variant:: Locked                        423  Locked                         
.. variant:: FailedDependency              424  Failed Dependency              
.. variant:: UnorderedCollection           425  Unordered Collection           
.. variant:: UpgradeRequired               426  Upgrade Required               
.. variant:: PreconditionRequired          428  Precondition Required          
.. variant:: TooManyRequests               429  Too Many Requests              
.. variant:: RequestHeaderFieldsTooLarge   431  Request Header Fields Too Large
.. variant:: UnavailableForLegalReasons    451  Unavailable For Legal Reasons  
.. variant:: InternalServerError           500  Internal Server Error          
.. variant:: NotImplemented                501  Not Implemented                
.. variant:: BadGateway                    502  Bad Gateway                    
.. variant:: ServiceUnavailable            503  Service Unavailable            
.. variant:: GatewayTimeout                504  Gateway Timeout                
.. variant:: HttpVersionNotSupported       505  HTTP Version Not Supported     
.. variant:: VariantAlsoNegotiates         506  Variant Also Negotiates        
.. variant:: InsufficientStorage           507  Insufficient Storage           
.. variant:: LoopDetected                  508  Loop Detected                  
.. variant:: NotExtended                   510  Not Extended                   
.. variant:: NetworkAuthenticationRequired 511  Network Authentication Required
========================================== ==== ===============================

There are also many other status codes representing the other variants in this
500-variant enum; they follow the form ``CodeNNN`` (for the three digits NNN)
and are listed below.

.. method:: StatusCode.canonical_reason(&self) -> Option<&'static str>

   Get the standardised ``Reason-Phrase`` for this status code.
   
   :returns: ``Some`` of the value from the table above for registered values,
             ``None`` for unregistered variants (those following the
             ``CodeNNN`` convention).

   This is mostly here for servers writing responses, but could potentially
   have application at other times.

   The reason phrase is defined as being exclusively for human readers. You
   should avoid deriving any meaning from it at all costs.

   Bear in mind also that in HTTP/2.0 the reason phrase is abolished from
   transmission, and so this canonical reason phrase really is the only reason
   phrase you'll find.

   Sample usage::

      >>> use httpcommon::status::{Code123, ImATeapot};  // hide
      >>> Code123.canonical_reason()
      None
      >>> ImATeapot.canonical_reason()
      Some(I'm a teapot)

.. method:: StatusCode.class(&self) -> StatusClass

   Determine the class of a status code, based on its first digit.

.. _httpcommon-status-statusclass:

:enum:`StatusClass` --- the class of an HTTP Status-Code
--------------------------------------------------------

.. enum:: StatusClass

   .. variant:: Informational = 100

      1xx: Informational - Request received, continuing process

   .. variant:: Success = 200
   
      2xx: Success - The action was successfully received, understood, and
      accepted

   .. variant:: Redirection = 300

      3xx: Redirection - Further action must be taken in order to complete the
      request

   .. variant:: ClientError = 400

      4xx: Client Error - The request contains bad syntax or cannot be
      fulfilled

   .. variant:: ServerError = 500

      5xx: Server Error - The server failed to fulfill an apparently valid
      request

`RFC 2616, section 6.1.1 (Status Code and Reason Phrase) <rfc2616-6.1.1>`_:

   The first digit of the Status-Code defines the class of response. The
   last two digits do not have any categorization role.
   
   ...
   
   HTTP status codes are extensible. HTTP applications are not required
   to understand the meaning of all registered status codes, though such
   understanding is obviously desirable. However, applications MUST
   understand the class of any status code, as indicated by the first
   digit, and treat any unrecognized response as being equivalent to the
   x00 status code of that class, with the exception that an
   unrecognized response MUST NOT be cached. For example, if an
   unrecognized status code of 431 is received by the client, it can
   safely assume that there was something wrong with its request and
   treat the response as if it had received a 400 status code. In such
   cases, user agents SHOULD present to the user the entity returned
   with the response, since that entity is likely to include human-
   readable information which will explain the unusual status.

This can be used in cases where a status code's meaning is unknown, also,
to get the appropriate *category* of status.

For HTTP/2.0, the 1xx Informational class is invalid.

.. method:: StatusClass.default_code(&self) -> StatusCode

   Get the default status code for the class.

   This produces the x00 status code; thus, for `ClientError` (4xx), for
   example, this will produce `BadRequest` (400)::

      >>> use httpcommon::status::ClientError;  // hide
      >>> ClientError.default_code()
      400 Bad Request

   The use for this is outlined in RFC 2616, section 6.1.1, as quoted earlier.

   This is demonstrated thusly (I'll use 432 rather than 431 as 431 *is* now in
   use):

      >>> use httpcommon::status::Code432;  // hide
      >>> // Suppose we have received this status code.
      >>> let status = Code432;
      >>> // Uh oh! Don't know what to do with it.
      >>> // Let's fall back to the default:
      >>> let status = status.class().default_code();
      >>> // Now see what it is; that's what we'll treat it as.
      >>> status
      400 Bad Request

.. _httpcommon-status-statuscode-other-variants:

The other :enum:`StatusCode` variants
-------------------------------------

For completeness, here are the remaining :enum:`StatusCode` variants. Generally
you shouldn't need to worry about them, but it's possible that you might at
some point need to. They're here when you need them.

.. currentenum:: status::StatusCode

.. variant:: Code103
.. variant:: Code104
.. variant:: Code105
.. variant:: Code106
.. variant:: Code107
.. variant:: Code108
.. variant:: Code109
.. variant:: Code110
.. variant:: Code111
.. variant:: Code112
.. variant:: Code113
.. variant:: Code114
.. variant:: Code115
.. variant:: Code116
.. variant:: Code117
.. variant:: Code118
.. variant:: Code119
.. variant:: Code120
.. variant:: Code121
.. variant:: Code122
.. variant:: Code123
.. variant:: Code124
.. variant:: Code125
.. variant:: Code126
.. variant:: Code127
.. variant:: Code128
.. variant:: Code129
.. variant:: Code130
.. variant:: Code131
.. variant:: Code132
.. variant:: Code133
.. variant:: Code134
.. variant:: Code135
.. variant:: Code136
.. variant:: Code137
.. variant:: Code138
.. variant:: Code139
.. variant:: Code140
.. variant:: Code141
.. variant:: Code142
.. variant:: Code143
.. variant:: Code144
.. variant:: Code145
.. variant:: Code146
.. variant:: Code147
.. variant:: Code148
.. variant:: Code149
.. variant:: Code150
.. variant:: Code151
.. variant:: Code152
.. variant:: Code153
.. variant:: Code154
.. variant:: Code155
.. variant:: Code156
.. variant:: Code157
.. variant:: Code158
.. variant:: Code159
.. variant:: Code160
.. variant:: Code161
.. variant:: Code162
.. variant:: Code163
.. variant:: Code164
.. variant:: Code165
.. variant:: Code166
.. variant:: Code167
.. variant:: Code168
.. variant:: Code169
.. variant:: Code170
.. variant:: Code171
.. variant:: Code172
.. variant:: Code173
.. variant:: Code174
.. variant:: Code175
.. variant:: Code176
.. variant:: Code177
.. variant:: Code178
.. variant:: Code179
.. variant:: Code180
.. variant:: Code181
.. variant:: Code182
.. variant:: Code183
.. variant:: Code184
.. variant:: Code185
.. variant:: Code186
.. variant:: Code187
.. variant:: Code188
.. variant:: Code189
.. variant:: Code190
.. variant:: Code191
.. variant:: Code192
.. variant:: Code193
.. variant:: Code194
.. variant:: Code195
.. variant:: Code196
.. variant:: Code197
.. variant:: Code198
.. variant:: Code199
.. variant:: Code209
.. variant:: Code210
.. variant:: Code211
.. variant:: Code212
.. variant:: Code213
.. variant:: Code214
.. variant:: Code215
.. variant:: Code216
.. variant:: Code217
.. variant:: Code218
.. variant:: Code219
.. variant:: Code220
.. variant:: Code221
.. variant:: Code222
.. variant:: Code223
.. variant:: Code224
.. variant:: Code225
.. variant:: Code227
.. variant:: Code228
.. variant:: Code229
.. variant:: Code230
.. variant:: Code231
.. variant:: Code232
.. variant:: Code233
.. variant:: Code234
.. variant:: Code235
.. variant:: Code236
.. variant:: Code237
.. variant:: Code238
.. variant:: Code239
.. variant:: Code240
.. variant:: Code241
.. variant:: Code242
.. variant:: Code243
.. variant:: Code244
.. variant:: Code245
.. variant:: Code246
.. variant:: Code247
.. variant:: Code248
.. variant:: Code249
.. variant:: Code250
.. variant:: Code251
.. variant:: Code252
.. variant:: Code253
.. variant:: Code254
.. variant:: Code255
.. variant:: Code256
.. variant:: Code257
.. variant:: Code258
.. variant:: Code259
.. variant:: Code260
.. variant:: Code261
.. variant:: Code262
.. variant:: Code263
.. variant:: Code264
.. variant:: Code265
.. variant:: Code266
.. variant:: Code267
.. variant:: Code268
.. variant:: Code269
.. variant:: Code270
.. variant:: Code271
.. variant:: Code272
.. variant:: Code273
.. variant:: Code274
.. variant:: Code275
.. variant:: Code276
.. variant:: Code277
.. variant:: Code278
.. variant:: Code279
.. variant:: Code280
.. variant:: Code281
.. variant:: Code282
.. variant:: Code283
.. variant:: Code284
.. variant:: Code285
.. variant:: Code286
.. variant:: Code287
.. variant:: Code288
.. variant:: Code289
.. variant:: Code290
.. variant:: Code291
.. variant:: Code292
.. variant:: Code293
.. variant:: Code294
.. variant:: Code295
.. variant:: Code296
.. variant:: Code297
.. variant:: Code298
.. variant:: Code299
.. variant:: Code309
.. variant:: Code310
.. variant:: Code311
.. variant:: Code312
.. variant:: Code313
.. variant:: Code314
.. variant:: Code315
.. variant:: Code316
.. variant:: Code317
.. variant:: Code318
.. variant:: Code319
.. variant:: Code320
.. variant:: Code321
.. variant:: Code322
.. variant:: Code323
.. variant:: Code324
.. variant:: Code325
.. variant:: Code326
.. variant:: Code327
.. variant:: Code328
.. variant:: Code329
.. variant:: Code330
.. variant:: Code331
.. variant:: Code332
.. variant:: Code333
.. variant:: Code334
.. variant:: Code335
.. variant:: Code336
.. variant:: Code337
.. variant:: Code338
.. variant:: Code339
.. variant:: Code340
.. variant:: Code341
.. variant:: Code342
.. variant:: Code343
.. variant:: Code344
.. variant:: Code345
.. variant:: Code346
.. variant:: Code347
.. variant:: Code348
.. variant:: Code349
.. variant:: Code350
.. variant:: Code351
.. variant:: Code352
.. variant:: Code353
.. variant:: Code354
.. variant:: Code355
.. variant:: Code356
.. variant:: Code357
.. variant:: Code358
.. variant:: Code359
.. variant:: Code360
.. variant:: Code361
.. variant:: Code362
.. variant:: Code363
.. variant:: Code364
.. variant:: Code365
.. variant:: Code366
.. variant:: Code367
.. variant:: Code368
.. variant:: Code369
.. variant:: Code370
.. variant:: Code371
.. variant:: Code372
.. variant:: Code373
.. variant:: Code374
.. variant:: Code375
.. variant:: Code376
.. variant:: Code377
.. variant:: Code378
.. variant:: Code379
.. variant:: Code380
.. variant:: Code381
.. variant:: Code382
.. variant:: Code383
.. variant:: Code384
.. variant:: Code385
.. variant:: Code386
.. variant:: Code387
.. variant:: Code388
.. variant:: Code389
.. variant:: Code390
.. variant:: Code391
.. variant:: Code392
.. variant:: Code393
.. variant:: Code394
.. variant:: Code395
.. variant:: Code396
.. variant:: Code397
.. variant:: Code398
.. variant:: Code399
.. variant:: Code420
.. variant:: Code421
.. variant:: Code427
.. variant:: Code430
.. variant:: Code432
.. variant:: Code433
.. variant:: Code434
.. variant:: Code435
.. variant:: Code436
.. variant:: Code437
.. variant:: Code438
.. variant:: Code439
.. variant:: Code440
.. variant:: Code441
.. variant:: Code442
.. variant:: Code443
.. variant:: Code444
.. variant:: Code445
.. variant:: Code446
.. variant:: Code447
.. variant:: Code448
.. variant:: Code449
.. variant:: Code450
.. variant:: Code452
.. variant:: Code453
.. variant:: Code454
.. variant:: Code455
.. variant:: Code456
.. variant:: Code457
.. variant:: Code458
.. variant:: Code459
.. variant:: Code460
.. variant:: Code461
.. variant:: Code462
.. variant:: Code463
.. variant:: Code464
.. variant:: Code465
.. variant:: Code466
.. variant:: Code467
.. variant:: Code468
.. variant:: Code469
.. variant:: Code470
.. variant:: Code471
.. variant:: Code472
.. variant:: Code473
.. variant:: Code474
.. variant:: Code475
.. variant:: Code476
.. variant:: Code477
.. variant:: Code478
.. variant:: Code479
.. variant:: Code480
.. variant:: Code481
.. variant:: Code482
.. variant:: Code483
.. variant:: Code484
.. variant:: Code485
.. variant:: Code486
.. variant:: Code487
.. variant:: Code488
.. variant:: Code489
.. variant:: Code490
.. variant:: Code491
.. variant:: Code492
.. variant:: Code493
.. variant:: Code494
.. variant:: Code495
.. variant:: Code496
.. variant:: Code497
.. variant:: Code498
.. variant:: Code499
.. variant:: Code509
.. variant:: Code512
.. variant:: Code513
.. variant:: Code514
.. variant:: Code515
.. variant:: Code516
.. variant:: Code517
.. variant:: Code518
.. variant:: Code519
.. variant:: Code520
.. variant:: Code521
.. variant:: Code522
.. variant:: Code523
.. variant:: Code524
.. variant:: Code525
.. variant:: Code526
.. variant:: Code527
.. variant:: Code528
.. variant:: Code529
.. variant:: Code530
.. variant:: Code531
.. variant:: Code532
.. variant:: Code533
.. variant:: Code534
.. variant:: Code535
.. variant:: Code536
.. variant:: Code537
.. variant:: Code538
.. variant:: Code539
.. variant:: Code540
.. variant:: Code541
.. variant:: Code542
.. variant:: Code543
.. variant:: Code544
.. variant:: Code545
.. variant:: Code546
.. variant:: Code547
.. variant:: Code548
.. variant:: Code549
.. variant:: Code550
.. variant:: Code551
.. variant:: Code552
.. variant:: Code553
.. variant:: Code554
.. variant:: Code555
.. variant:: Code556
.. variant:: Code557
.. variant:: Code558
.. variant:: Code559
.. variant:: Code560
.. variant:: Code561
.. variant:: Code562
.. variant:: Code563
.. variant:: Code564
.. variant:: Code565
.. variant:: Code566
.. variant:: Code567
.. variant:: Code568
.. variant:: Code569
.. variant:: Code570
.. variant:: Code571
.. variant:: Code572
.. variant:: Code573
.. variant:: Code574
.. variant:: Code575
.. variant:: Code576
.. variant:: Code577
.. variant:: Code578
.. variant:: Code579
.. variant:: Code580
.. variant:: Code581
.. variant:: Code582
.. variant:: Code583
.. variant:: Code584
.. variant:: Code585
.. variant:: Code586
.. variant:: Code587
.. variant:: Code588
.. variant:: Code589
.. variant:: Code590
.. variant:: Code591
.. variant:: Code592
.. variant:: Code593
.. variant:: Code594
.. variant:: Code595
.. variant:: Code596
.. variant:: Code597
.. variant:: Code598
.. variant:: Code599

.. _rfc2616-6.1.1: https://tools.ietf.org/html/rfc2616#section-6.1.1
