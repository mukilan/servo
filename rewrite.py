import ast_grep_py as ag

files = [
"components/script/dom/htmltimeelement.rs",
"components/script/dom/htmltitleelement.rs",
"components/script/dom/htmltrackelement.rs",
"components/script/dom/htmlulistelement.rs",
"components/script/dom/htmlunknownelement.rs",
"components/script/dom/htmlvideoelement.rs",
"components/script/dom/htmlaudioelement.rs",
"components/script/dom/htmlbaseelement.rs",
"components/script/dom/htmlbodyelement.rs",
"components/script/dom/htmlbrelement.rs",
#"components/script/dom/htmlbuttonelement.rs",
"components/script/dom/htmlcanvaselement.rs",
"components/script/dom/htmldataelement.rs",
"components/script/dom/htmldatalistelement.rs",
"components/script/dom/htmldetailselement.rs",
"components/script/dom/htmldialogelement.rs",
"components/script/dom/htmldirectoryelement.rs",
"components/script/dom/htmldivelement.rs",
"components/script/dom/htmldlistelement.rs",
"components/script/dom/htmlembedelement.rs",
"components/script/dom/htmlfontelement.rs",
#"components/script/dom/htmlformelement.rs",
"components/script/dom/htmlframeelement.rs",
"components/script/dom/htmlframesetelement.rs",
"components/script/dom/htmlheadelement.rs",
"components/script/dom/htmlheadingelement.rs",
"components/script/dom/htmlhrelement.rs",
"components/script/dom/htmliframeelement.rs",
"components/script/dom/htmlimageelement.rs",
#"components/script/dom/htmlinputelement.rs",
"components/script/dom/htmllabelelement.rs",
"components/script/dom/htmllegendelement.rs",
"components/script/dom/htmllielement.rs",
"components/script/dom/htmllinkelement.rs",
"components/script/dom/htmlmapelement.rs",
#"components/script/dom/htmlmediaelement.rs",
"components/script/dom/htmlmenuelement.rs",
"components/script/dom/htmlmetaelement.rs",
"components/script/dom/htmlmeterelement.rs",
"components/script/dom/htmlmodelement.rs",
"components/script/dom/htmlobjectelement.rs",
"components/script/dom/htmlolistelement.rs",
#"components/script/dom/htmloptgroupelement.rs",
#"components/script/dom/htmloptionelement.rs",
"components/script/dom/htmloutputelement.rs",
"components/script/dom/htmlparagraphelement.rs",
"components/script/dom/htmlparamelement.rs",
"components/script/dom/htmlpictureelement.rs",
"components/script/dom/htmlpreelement.rs",
"components/script/dom/htmlprogresselement.rs",
"components/script/dom/htmlquoteelement.rs",
"components/script/dom/htmlscriptelement.rs",
#"components/script/dom/htmlselectelement.rs",
"components/script/dom/htmlslotelement.rs",
"components/script/dom/htmlsourceelement.rs",
"components/script/dom/htmlspanelement.rs",
"components/script/dom/htmlstyleelement.rs",
"components/script/dom/htmltablecaptionelement.rs",
"components/script/dom/htmltablecellelement.rs",
"components/script/dom/htmltablecolelement.rs",
"components/script/dom/htmltableelement.rs",
"components/script/dom/htmltablerowelement.rs",
"components/script/dom/htmltablesectionelement.rs",
"components/script/dom/htmltemplateelement.rs",
#"components/script/dom/htmltextareaelement.rs",
"components/script/dom/htmltimeelement.rs",
"components/script/dom/htmltitleelement.rs",
"components/script/dom/htmltrackelement.rs",
"components/script/dom/htmlulistelement.rs",
"components/script/dom/htmlunknownelement.rs",
"components/script/dom/htmlvideoelement.rs",
]

def matches_str(matches):
    return "".join([x.text() for x in matches])


for file in files:
    try:
        with open(file, 'r+') as src_file:
            src_code = src_file.read()
            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(pattern={
                "context": "impl $_ {  pub(crate) fn new($$$ARGS) ->  $RET { $$$BODY } }",
                "selector": "function_item",
            })

            new_call = newfn.find(pattern="$T::new_inherited($$$ARGS)")
            new_call_args = []
            for (i, a) in enumerate(new_call.get_multiple_matches("ARGS")):
                if a.kind() == "identifier":
                    new_call_args.append(a.text())
            new_call_args.append("is_defined")
            new_call_args = ", ".join(new_call_args) + ","
            call_edit = new_call.replace(f"{new_call['T'].text()}::new_inherited({new_call_args})")
            src_code = root.root().commit_edits([call_edit])

            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(pattern={
                "context": "impl $_ {  pub(crate) fn new($$$ARGS) ->  $RET { $$$BODY } }",
                "selector": "function_item",
            })


            args = newfn.get_multiple_matches("ARGS")
            new_args = []
            for (i, a) in enumerate(args):
                if a.kind() == "parameter":
                    new_args.append(a.text())

            # print(new_args)

            new_args.insert(-1, "is_defined: bool")
            replfn = f"""pub(crate) fn new({", ".join(new_args) + ","}) -> {newfn.get_match("RET").text()} {{
                {matches_str(newfn.get_multiple_matches("BODY"))}
            }}
            """

            edit = newfn.replace(replfn)
            #print(root.root().commit_edits([edit]))
            src_code = root.root().commit_edits([edit])


            root = ag.SgRoot(src_code, "rust")
            newfn = root.root().find(pattern={
                "context": "impl $_ {  fn new_inherited($$$ARGS) ->  $RET { $$$BODY } }",
                "selector": "function_item",
            })

            args = newfn.get_multiple_matches("ARGS")
            new_args = []
            for (i, a) in enumerate(args):
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
            newfn = root.root().find(pattern={
                "context": "impl $_ {  fn new_inherited($$$ARGS) ->  $RET { $$$BODY } }",
                "selector": "function_item",
            })

            new_call = newfn.find(pattern="$T::new_inherited($$$ARGS)")
            new_call_args = []
            for (i, a) in enumerate(new_call.get_multiple_matches("ARGS")):
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

