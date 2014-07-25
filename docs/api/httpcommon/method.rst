:mod:`method` --- HTTP methods
==============================

.. module:: method

Module contents:

- :enum:`Method` --- HTTP method

.. _httpcommon-method-method:

:enum:`Method` --- HTTP method
------------------------------

.. enum:: Method

   (Quite a few variants, omitted here for easier comprehension.)

An HTTP method (``method`` in :rfc:`7230` and :rfc:`7231`).

… explanation of why and how and so forth …

============================== ===================== ==== ========== ===================================================
Variant                        Method name           Safe Idempotent Defined in
============================== ===================== ==== ========== ===================================================
.. variant:: Acl               ``ACL``               no   yes        :rfc:`3744#section-8.1`
.. variant:: BaselineControl   ``BASELINE-CONTROL``  no   yes        :rfc:`3253#section-12.6`
.. variant:: Bind              ``BIND``              no   yes        :rfc:`5842#section-4`
.. variant:: Checkin           ``CHECKIN``           no   yes        :rfc:`3253#section-4.4` and :rfc:`3253#section-9.4`
.. variant:: Checkout          ``CHECKOUT``          no   yes        :rfc:`3253#section-4.3` and :rfc:`3253#section-8.8`
.. variant:: Connect           ``CONNECT``           no   no         :rfc:`7231#section-4.3.6`
.. variant:: Copy              ``COPY``              no   yes        :rfc:`4918#section-9.8`
.. variant:: Delete            ``DELETE``            no   yes        :rfc:`7231#section-4.3.5`
.. variant:: Get               ``GET``               yes  yes        :rfc:`7231#section-4.3.1`
.. variant:: Head              ``HEAD``              yes  yes        :rfc:`7231#section-4.3.2`
.. variant:: Label             ``LABEL``             no   yes        :rfc:`3253#section-8.2`
.. variant:: Link              ``LINK``              no   yes        :rfc:`2068#section-19.6.1.2`
.. variant:: Lock              ``LOCK``              no   no         :rfc:`4918#section-9.10`
.. variant:: Merge             ``MERGE``             no   yes        :rfc:`3253#section-11.2`
.. variant:: MkActivity        ``MKACTIVITY``        no   yes        :rfc:`3253#section-13.5`
.. variant:: MkCalendar        ``MKCALENDAR``        no   yes        :rfc:`4791#section-5.3.1`
.. variant:: MkCol             ``MKCOL``             no   yes        :rfc:`4918#section-9.3`
.. variant:: MkRedirectRef     ``MKREDIRECTREF``     no   yes        :rfc:`4437#section-6`
.. variant:: MkWorkspace       ``MKWORKSPACE``       no   yes        :rfc:`3253#section-6.3`
.. variant:: Move              ``MOVE``              no   yes        :rfc:`4918#section-9.9`
.. variant:: Options           ``OPTIONS``           yes  yes        :rfc:`7231#section-4.3.7`
.. variant:: OrderPatch        ``ORDERPATCH``        no   yes        :rfc:`3648#section-7`
.. variant:: Patch             ``PATCH``             no   no         :rfc:`5789#section-2`
.. variant:: Post              ``POST``              no   no         :rfc:`7231#section-4.3.3`
.. variant:: PropFind          ``PROPFIND``          yes  yes        :rfc:`4918#section-9.1`
.. variant:: PropPatch         ``PROPPATCH``         no   yes        :rfc:`4918#section-9.2`
.. variant:: Put               ``PUT``               no   yes        :rfc:`7231#section-4.3.4`
.. variant:: Rebind            ``REBIND``            no   yes        :rfc:`5842#section-6`
.. variant:: Report            ``REPORT``            yes  yes        :rfc:`3253#section-3.6`
.. variant:: Search            ``SEARCH``            yes  yes        :rfc:`5323#section-2`
.. variant:: Trace             ``TRACE``             yes  yes        :rfc:`7231#section-4.3.8`
.. variant:: Unbind            ``UNBIND``            no   yes        :rfc:`5842#section-5`
.. variant:: Uncheckout        ``UNCHECKOUT``        no   yes        :rfc:`3253#section-4.5`
.. variant:: Unlink            ``UNLINK``            no   yes        :rfc:`2068#section-19.6.1.3`
.. variant:: Unlock            ``UNLOCK``            no   yes        :rfc:`4918#section-9.11`
.. variant:: Update            ``UPDATE``            no   yes        :rfc:`3253#section-7.1`
.. variant:: UpdateRedirectRef ``UPDATEREDIRECTREF`` no   yes        :rfc:`4437#section-7`
.. variant:: VersionControl    ``VERSION-CONTROL``   no   yes        :rfc:`3253#section-3.5`
============================== ===================== ==== ========== ===================================================
