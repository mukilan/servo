function process_raw_results(raw_results) {
  const test_scores = []
  const test_names = new Set()
  const run_info = raw_results.run_info

  for (const test of raw_results.results) {
    const test_name = test.test;
    const test_status = test.status;

    test_names.add(test_name)
    test_scores.push({
      name: test_name,
      score: test_status === "PASS" ? 1 : 0
    })

    for (const subtest of test.subtests) {
      const subtest_name = `${test_name}::${subtest.name}`
      test_names.add(subtest_name)
      test_scores.push({
        name: subtest_name ,
        score: subtest.status === "PASS" ? 1 : 0 
      })
    }
  }

  return { run_info, test_names, test_scores }
}

const results = JSON.parse(process.argv[2]);
const processed_results = process_raw_results(results);
console.log(JSON.stringify(processed_results, ["run_info", "test_scores"]));
