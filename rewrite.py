import ast_grep_py as ag

files = [
    "components/script/dom/html/htmltimeelement.rs",
    "components/script/dom/html/htmltitleelement.rs",
    "components/script/dom/html/htmltrackelement.rs",
    "components/script/dom/html/htmlulistelement.rs",
    "components/script/dom/html/htmlunknownelement.rs",
    "components/script/dom/html/htmlvideoelement.rs",
    "components/script/dom/html/htmlaudioelement.rs",
    "components/script/dom/html/htmlbaseelement.rs",
    "components/script/dom/html/htmlbodyelement.rs",
    "components/script/dom/html/htmlbrelement.rs",
    "components/script/dom/html/htmlcanvaselement.rs",
    "components/script/dom/html/htmldataelement.rs",
    "components/script/dom/html/htmldatalistelement.rs",
    "components/script/dom/html/htmldetailselement.rs",
    "components/script/dom/html/htmldialogelement.rs",
    "components/script/dom/html/htmldirectoryelement.rs",
    "components/script/dom/html/htmldivelement.rs",
    "components/script/dom/html/htmldlistelement.rs",
    "components/script/dom/html/htmlembedelement.rs",
    "components/script/dom/html/htmlfontelement.rs",
    "components/script/dom/html/htmlframeelement.rs",
    "components/script/dom/html/htmlframesetelement.rs",
    "components/script/dom/html/htmlheadelement.rs",
    "components/script/dom/html/htmlheadingelement.rs",
    "components/script/dom/html/htmlhrelement.rs",
    "components/script/dom/html/htmliframeelement.rs",
    "components/script/dom/html/htmlimageelement.rs",
    "components/script/dom/html/htmllabelelement.rs",
    "components/script/dom/html/htmllegendelement.rs",
    "components/script/dom/html/htmllielement.rs",
    "components/script/dom/html/htmllinkelement.rs",
    "components/script/dom/html/htmlmapelement.rs",
    "components/script/dom/html/htmlmenuelement.rs",
    "components/script/dom/html/htmlmetaelement.rs",
    "components/script/dom/html/htmlmeterelement.rs",
    "components/script/dom/html/htmlmodelement.rs",
    "components/script/dom/html/htmlobjectelement.rs",
    "components/script/dom/html/htmlolistelement.rs",
    "components/script/dom/html/htmloutputelement.rs",
    "components/script/dom/html/htmlparagraphelement.rs",
    "components/script/dom/html/htmlparamelement.rs",
    "components/script/dom/html/htmlpictureelement.rs",
    "components/script/dom/html/htmlpreelement.rs",
    "components/script/dom/html/htmlprogresselement.rs",
    "components/script/dom/html/htmlquoteelement.rs",
    "components/script/dom/html/htmlscriptelement.rs",
    "components/script/dom/html/htmlslotelement.rs",
    "components/script/dom/html/htmlsourceelement.rs",
    "components/script/dom/html/htmlspanelement.rs",
    "components/script/dom/html/htmlstyleelement.rs",
    "components/script/dom/html/htmltablecaptionelement.rs",
    "components/script/dom/html/htmltablecellelement.rs",
    "components/script/dom/html/htmltablecolelement.rs",
    "components/script/dom/html/htmltableelement.rs",
    "components/script/dom/html/htmltablerowelement.rs",
    "components/script/dom/html/htmltablesectionelement.rs",
    "components/script/dom/html/htmltemplateelement.rs",
]


def matches_str(matches):
    return "".join([x.text() for x in matches])


for file in files:
    try:
        with open(file, "r+") as src_file:
            src_code = src_file.read()
            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(
                pattern={
                    "context": "impl $_ {  pub(crate) fn new($$$ARGS) ->  $RET { $$$BODY } }",
                    "selector": "function_item",
                }
            )

            new_call = newfn.find(pattern="$T::new_inherited($$$ARGS)")
            new_call_args = []
            for i, a in enumerate(new_call.get_multiple_matches("ARGS")):
                if a.kind() == "identifier":
                    new_call_args.append(a.text())
            new_call_args.append("is_defined")
            new_call_args = ", ".join(new_call_args) + ","
            call_edit = new_call.replace(f"{new_call['T'].text()}::new_inherited({new_call_args})")
            src_code = root.root().commit_edits([call_edit])

            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(
                pattern={
                    "context": "impl $_ {  pub(crate) fn new($$$ARGS) ->  $RET { $$$BODY } }",
                    "selector": "function_item",
                }
            )

            args = newfn.get_multiple_matches("ARGS")
            new_args = []
            for i, a in enumerate(args):
                if a.kind() == "parameter":
                    new_args.append(a.text())

            # print(new_args)

            new_args.insert(-1, "is_defined: bool")
            replfn = f"""pub(crate) fn new({", ".join(new_args) + ","}) -> {newfn.get_match("RET").text()} {{
                {matches_str(newfn.get_multiple_matches("BODY"))}
            }}
            """

            edit = newfn.replace(replfn)
            # print(root.root().commit_edits([edit]))
            src_code = root.root().commit_edits([edit])

            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(
                pattern={
                    "context": "impl $_ {  fn new_inherited($$$ARGS) ->  $RET { $$$BODY } }",
                    "selector": "function_item",
                }
            )

            args = newfn.get_multiple_matches("ARGS")
            new_args = []
            for i, a in enumerate(args):
                if a.kind() == "parameter":
                    new_args.append(a.text())

            # print(new_args)

            new_args.append("is_defined: bool")
            replfn = f"""fn new_inherited({", ".join(new_args) + ","}) -> {newfn.get_match("RET").text()} {{
                {matches_str(newfn.get_multiple_matches("BODY"))}
            }}
            """

            edit = newfn.replace(replfn)
            src_code = root.root().commit_edits([edit])
            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(
                pattern={
                    "context": "impl $_ {  fn new_inherited($$$ARGS) ->  $RET { $$$BODY } }",
                    "selector": "function_item",
                }
            )

            new_call = newfn.find(pattern="$T::new_inherited($$$ARGS)")
            new_call_args = []
            for i, a in enumerate(new_call.get_multiple_matches("ARGS")):
                if a.kind() == "identifier":
                    new_call_args.append(a.text())
            new_call_args.append("is_defined")
            new_call_args = ", ".join(new_call_args) + ","
            call_edit = new_call.replace(f"{new_call['T'].text()}::new_inherited({new_call_args})")
            src_code = root.root().commit_edits([call_edit])

            src_file.seek(0)
            src_file.write(src_code)
            src_file.truncate()
    except Exception as e:
        print("Unable to process file", file, "\n due to ", e)
