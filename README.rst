Common HTTP functionality for the Teepee project
================================================

.. image:: https://travis-ci.org/teepee/httpcommon.png?branch=master
   :target: https://travis-ci.org/teepee/httpcommon

Anything shared between both client and server belongs in here, but this crate
is not expected to be used directly.

Any crate using types from this crate should re‐export them. For example, the
``status`` module should be exported in the root of the HTTP client crate
``httpc`` so that people can write ``httpc::status`` instead of
``httpcommon::status``.

Author
------

`Chris Morgan <http://chrismorgan.info>`_ (`chris-morgan
<https://github.com/chris-morgan>`_) is the primary author and maintainer of
this crate.

License
-------

This library is distributed under similar terms to Rust: dual licensed under
the MIT license and the Apache license (version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
