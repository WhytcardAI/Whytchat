/**
 * WhytChat Code Analysis Report
 * ==============================
 * Analyse statique du code pour identifier les problèmes potentiels
 */

const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, '..', 'src');

// Patterns à rechercher
const patterns = {
  // Dangerous patterns
  dangerous: [
    { name: 'eval usage', regex: /\beval\s*\(/g, severity: 'CRITICAL' },
    { name: 'dangerouslySetInnerHTML', regex: /dangerouslySetInnerHTML/g, severity: 'HIGH' },
    { name: 'innerHTML assignment', regex: /\.innerHTML\s*=/g, severity: 'HIGH' },
    { name: 'document.write', regex: /document\.write\s*\(/g, severity: 'CRITICAL' },
    { name: 'Function constructor', regex: /new\s+Function\s*\(/g, severity: 'HIGH' },
  ],
  
  // Potential issues
  warnings: [
    { name: 'console.log (production)', regex: /console\.log\s*\(/g, severity: 'LOW' },
    { name: 'console.error without logger', regex: /console\.error\s*\(/g, severity: 'LOW' },
    { name: 'setTimeout without cleanup', regex: /setTimeout\s*\([^)]+\)\s*(?!.*clearTimeout)/g, severity: 'MEDIUM' },
    { name: 'setInterval without cleanup', regex: /setInterval\s*\([^)]+\)\s*(?!.*clearInterval)/g, severity: 'HIGH' },
    { name: 'TODO comments', regex: /\/\/\s*TODO/gi, severity: 'INFO' },
    { name: 'FIXME comments', regex: /\/\/\s*FIXME/gi, severity: 'MEDIUM' },
    { name: 'HACK comments', regex: /\/\/\s*HACK/gi, severity: 'MEDIUM' },
  ],
  
  // Best practices
  practices: [
    { name: 'useEffect without deps array', regex: /useEffect\s*\(\s*\(\)\s*=>\s*\{[^}]*\}\s*\)/g, severity: 'MEDIUM' },
    { name: 'any type usage', regex: /:\s*any\b/g, severity: 'LOW' },
    { name: 'eslint-disable comment', regex: /eslint-disable/g, severity: 'INFO' },
    { name: 'ts-ignore comment', regex: /@ts-ignore/g, severity: 'MEDIUM' },
  ],
  
  // Security
  security: [
    { name: 'hardcoded secret', regex: /(password|secret|api[_-]?key|token)\s*[:=]\s*['"][^'"]+['"]/gi, severity: 'CRITICAL' },
    { name: 'http:// URL', regex: /['"]http:\/\/(?!localhost)/g, severity: 'MEDIUM' },
    { name: 'file:// URL', regex: /['"]file:\/\//g, severity: 'HIGH' },
  ],
  
  // React specific
  react: [
    { name: 'Missing key in map', regex: /\.map\s*\([^)]*\)\s*=>\s*<[^>]+(?!key=)/g, severity: 'MEDIUM' },
    { name: 'Direct state mutation', regex: /state\.[a-zA-Z]+\s*=/g, severity: 'HIGH' },
    { name: 'useCallback missing deps', regex: /useCallback\s*\([^)]+\)\s*(?!\s*,\s*\[)/g, severity: 'MEDIUM' },
    { name: 'useMemo missing deps', regex: /useMemo\s*\([^)]+\)\s*(?!\s*,\s*\[)/g, severity: 'MEDIUM' },
  ]
};

// Files to analyze
const fileExtensions = ['.js', '.jsx', '.ts', '.tsx'];

function getAllFiles(dir, files = []) {
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        if (!entry.name.startsWith('.') && entry.name !== 'node_modules') {
          getAllFiles(fullPath, files);
        }
      } else if (fileExtensions.includes(path.extname(entry.name))) {
        files.push(fullPath);
      }
    }
  } catch (e) {
    // Directory doesn't exist or permission error
  }
  
  return files;
}

function analyzeFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const relativePath = path.relative(srcDir, filePath);
  const issues = [];
  
  for (const [category, patternList] of Object.entries(patterns)) {
    for (const pattern of patternList) {
      const matches = content.match(pattern.regex);
      if (matches && matches.length > 0) {
        // Get line numbers
        const lines = content.split('\n');
        const matchLines = [];
        lines.forEach((line, idx) => {
          if (pattern.regex.test(line)) {
            matchLines.push(idx + 1);
          }
          pattern.regex.lastIndex = 0; // Reset regex
        });
        
        issues.push({
          file: relativePath,
          category,
          pattern: pattern.name,
          severity: pattern.severity,
          count: matches.length,
          lines: matchLines.slice(0, 5), // First 5 occurrences
        });
      }
    }
  }
  
  return issues;
}

function generateReport() {
  console.log('╔═══════════════════════════════════════════════════════════════╗');
  console.log('║           WhytChat Static Code Analysis Report                ║');
  console.log('╚═══════════════════════════════════════════════════════════════╝\n');
  
  const files = getAllFiles(srcDir);
  console.log(`📁 Analyzing ${files.length} files in ${srcDir}\n`);
  
  const allIssues = [];
  
  for (const file of files) {
    const issues = analyzeFile(file);
    allIssues.push(...issues);
  }
  
  // Group by severity
  const bySeverity = {
    CRITICAL: [],
    HIGH: [],
    MEDIUM: [],
    LOW: [],
    INFO: []
  };
  
  allIssues.forEach(issue => {
    bySeverity[issue.severity].push(issue);
  });
  
  // Print by severity
  const severityIcons = {
    CRITICAL: '🔴',
    HIGH: '🟠',
    MEDIUM: '🟡',
    LOW: '🟢',
    INFO: '🔵'
  };
  
  for (const [severity, issues] of Object.entries(bySeverity)) {
    if (issues.length > 0) {
      console.log(`\n${severityIcons[severity]} ${severity} (${issues.length} issues):`);
      console.log('─'.repeat(50));
      
      issues.forEach(issue => {
        console.log(`  📄 ${issue.file}`);
        console.log(`     └─ ${issue.pattern} (×${issue.count}) [${issue.category}]`);
        if (issue.lines.length > 0) {
          console.log(`        Lines: ${issue.lines.join(', ')}${issue.lines.length < issue.count ? '...' : ''}`);
        }
      });
    }
  }
  
  // Summary
  console.log('\n╔═══════════════════════════════════════════════════════════════╗');
  console.log('║                         SUMMARY                               ║');
  console.log('╚═══════════════════════════════════════════════════════════════╝');
  console.log(`\n🔴 Critical: ${bySeverity.CRITICAL.length}`);
  console.log(`🟠 High:     ${bySeverity.HIGH.length}`);
  console.log(`🟡 Medium:   ${bySeverity.MEDIUM.length}`);
  console.log(`🟢 Low:      ${bySeverity.LOW.length}`);
  console.log(`🔵 Info:     ${bySeverity.INFO.length}`);
  console.log(`\n📊 Total:    ${allIssues.length} issues in ${files.length} files`);
  
  // Recommendations
  if (bySeverity.CRITICAL.length > 0) {
    console.log('\n⚠️  CRITICAL ISSUES REQUIRE IMMEDIATE ATTENTION!');
    console.log('   These patterns can lead to security vulnerabilities.\n');
  }
  
  return {
    files: files.length,
    issues: allIssues.length,
    bySeverity: Object.fromEntries(
      Object.entries(bySeverity).map(([k, v]) => [k, v.length])
    )
  };
}

// Run if executed directly
if (require.main === module) {
  try {
    generateReport();
  } catch (e) {
    console.error('Error running analysis:', e.message);
    console.log('\nMake sure you run this from the tests directory:');
    console.log('  cd apps/desktop-ui/tests && node code-analysis.js');
  }
}

module.exports = { generateReport, analyzeFile };
