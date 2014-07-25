"""
Sphinx plugins for documentation of Rust projects.
"""

from sphinx import addnodes

def setup(app):
    # Poor man's approach, definitely inferior to a proper Sphinx domain.
    # But it'll do for the moment; it's merely proof of concept.
    for prefix, directivename, rolename, indextemplate in (
            ('crate', 'crate', 'crate', 'pair: %s; crate'),
            ('mod', 'module', 'mod', 'pair: %s; module'),
            ('struct', 'struct', 'struct', 'pair: %s; struct'),
            ('enum', 'enum', 'enum', 'pair: %s; enum'),
            ('', 'variant', 'evar', 'pair: %s; enum variant'),
            ('type', 'type', 'type', 'pair: %s; type alias'),
            ('static', 'static', 'static', 'pair: %s; static'),
            ):
        app.add_object_type(directivename=directivename,
                            rolename=rolename,
                            indextemplate=indextemplate,
                            parse_node=parse_lang_node_maker(prefix))


def parse_lang_node_maker(prefix):
    def parse_lang_node(env, sig, signode):
        if prefix:
            title = "{} {}".format(prefix, sig)
        else:
            title = sig
        signode += addnodes.desc_name(title, title)
        return sig
    return parse_lang_node
