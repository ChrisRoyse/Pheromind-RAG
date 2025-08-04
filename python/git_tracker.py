#!/usr/bin/env python3
"""
Git Integration and Change Tracking for MCP RAG Indexer
Tracks git repository changes to optimize re-indexing
"""

import argparse
import json
import os
import sys
import subprocess
import logging
import time
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
import re

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Version information
VERSION = "1.0.0"

# Default code file extensions for filtering
CODE_EXTENSIONS = {
    'py', 'js', 'ts', 'java', 'cpp', 'c', 'h', 'hpp', 'cs', 'php', 
    'rb', 'go', 'rs', 'swift', 'kt', 'scala', 'm', 'mm', 'pl', 'r',
    'sql', 'html', 'css', 'xml', 'json', 'yaml', 'yml', 'md'
}


class GitRepository:
    """Represents a git repository and provides git operations"""
    
    def __init__(self, repo_path: str):
        self.repo_path = Path(repo_path).absolute()
        self.git_root = None
        self.logger = logging.getLogger(__name__)
        
        # Find git root
        self._find_git_root()
    
    def _find_git_root(self):
        """Find the root of the git repository"""
        current = self.repo_path
        
        while current != current.parent:
            git_dir = current / '.git'
            if git_dir.exists():
                self.git_root = current
                return
            current = current.parent
        
        # Not a git repository
        self.git_root = None
    
    def is_git_repo(self) -> bool:
        """Check if this is a git repository"""
        return self.git_root is not None
    
    def run_git_command(self, args: List[str], check: bool = True) -> subprocess.CompletedProcess:
        """Run a git command and return the result"""
        if not self.is_git_repo():
            raise RuntimeError("Not a git repository")
        
        cmd = ['git'] + args
        
        try:
            result = subprocess.run(
                cmd,
                cwd=str(self.git_root),
                capture_output=True,
                text=True,
                check=check,
                timeout=30
            )
            return result
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Git command failed: {' '.join(cmd)}")
            self.logger.error(f"Error: {e.stderr}")
            raise
        except subprocess.TimeoutExpired:
            self.logger.error(f"Git command timed out: {' '.join(cmd)}")
            raise
    
    def get_current_branch(self) -> Optional[str]:
        """Get the current branch name"""
        if not self.is_git_repo():
            return None
        
        try:
            result = self.run_git_command(['branch', '--show-current'])
            return result.stdout.strip() or None
        except:
            return None
    
    def get_current_commit(self) -> Optional[str]:
        """Get the current commit hash"""
        if not self.is_git_repo():
            return None
        
        try:
            result = self.run_git_command(['rev-parse', 'HEAD'])
            return result.stdout.strip()
        except:
            return None
    
    def get_status(self) -> Dict[str, List[str]]:
        """Get git status (modified, untracked files)"""
        if not self.is_git_repo():
            return {'modified_files': [], 'untracked_files': [], 'staged_files': []}
        
        try:
            result = self.run_git_command(['status', '--porcelain'])
            
            modified_files = []
            untracked_files = []
            staged_files = []
            
            for line in result.stdout.split('\n'):
                line = line.strip()
                if not line:
                    continue
                
                status = line[:2]
                filepath = line[3:]
                
                # Convert to absolute path
                abs_filepath = str(self.git_root / filepath)
                
                if status.startswith('??'):
                    untracked_files.append(abs_filepath)
                elif status.startswith('M ') or status.startswith(' M'):
                    modified_files.append(abs_filepath)
                elif status.startswith('A ') or status.startswith(' A'):
                    staged_files.append(abs_filepath)
                elif status.startswith('MM'):
                    modified_files.append(abs_filepath)
                    staged_files.append(abs_filepath)
            
            return {
                'modified_files': modified_files,
                'untracked_files': untracked_files,
                'staged_files': staged_files
            }
        except Exception as e:
            self.logger.error(f"Failed to get git status: {e}")
            return {'modified_files': [], 'untracked_files': [], 'staged_files': []}
    
    def get_commit_history(self, limit: int = 10) -> List[Dict[str, Any]]:
        """Get commit history"""
        if not self.is_git_repo():
            return []
        
        try:
            # Format: hash|author|date|message
            result = self.run_git_command([
                'log', f'-{limit}', '--pretty=format:%H|%an|%ad|%s', '--date=iso'
            ])
            
            commits = []
            for line in result.stdout.split('\n'):
                line = line.strip()
                if not line:
                    continue
                
                parts = line.split('|', 3)
                if len(parts) == 4:
                    commits.append({
                        'hash': parts[0],
                        'author': parts[1],
                        'date': parts[2],
                        'message': parts[3]
                    })
            
            return commits
        except Exception as e:
            self.logger.error(f"Failed to get commit history: {e}")
            return []
    
    def get_diff(self, commit_hash: str) -> List[Dict[str, Any]]:
        """Get diff since a commit"""
        if not self.is_git_repo():
            return []
        
        try:
            result = self.run_git_command([
                'diff', '--name-status', commit_hash + '..HEAD'
            ])
            
            changes = []
            for line in result.stdout.split('\n'):
                line = line.strip()
                if not line:
                    continue
                
                parts = line.split('\t', 1)
                if len(parts) == 2:
                    status_code = parts[0]
                    filepath = parts[1]
                    
                    # Map status codes
                    status_map = {
                        'A': 'added',
                        'M': 'modified',
                        'D': 'deleted',
                        'R': 'renamed',
                        'C': 'copied'
                    }
                    
                    status = status_map.get(status_code[0], 'unknown')
                    
                    changes.append({
                        'file': str(self.git_root / filepath),
                        'status': status,
                        'status_code': status_code
                    })
            
            return changes
        except Exception as e:
            self.logger.error(f"Failed to get diff: {e}")
            return []
    
    def get_branches(self) -> List[Dict[str, Any]]:
        """Get all branches"""
        if not self.is_git_repo():
            return []
        
        try:
            result = self.run_git_command(['branch', '-a'])
            
            branches = []
            current_branch = None
            
            for line in result.stdout.split('\n'):
                line = line.strip()
                if not line:
                    continue
                
                is_current = line.startswith('* ')
                if is_current:
                    line = line[2:]
                    current_branch = line
                
                # Skip remote tracking info
                if line.startswith('remotes/'):
                    continue
                
                branches.append({
                    'name': line,
                    'is_current': is_current
                })
            
            return branches
        except Exception as e:
            self.logger.error(f"Failed to get branches: {e}")
            return []
    
    def get_ignored_patterns(self) -> List[str]:
        """Get gitignore patterns"""
        patterns = []
        
        gitignore_path = self.git_root / '.gitignore'
        if gitignore_path.exists():
            try:
                with open(gitignore_path, 'r') as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith('#'):
                            patterns.append(line)
            except Exception as e:
                self.logger.warning(f"Failed to read .gitignore: {e}")
        
        return patterns
    
    def is_ignored(self, filepath: str) -> bool:
        """Check if a file would be ignored by git"""
        if not self.is_git_repo():
            return False
        
        try:
            # Use git check-ignore to test
            result = self.run_git_command(['check-ignore', filepath], check=False)
            return result.returncode == 0
        except:
            return False


class GitTracker:
    """Main git tracking and integration class"""
    
    def __init__(self, log_level: str = "info"):
        self.setup_logging(log_level)
        self.logger = logging.getLogger(__name__)
    
    def setup_logging(self, log_level: str):
        """Setup logging configuration"""
        level_map = {
            'debug': logging.DEBUG,
            'info': logging.INFO,
            'warning': logging.WARNING,
            'error': logging.ERROR
        }
        
        level = level_map.get(log_level.lower(), logging.INFO)
        logging.basicConfig(level=level, format='%(levelname)s: %(message)s')
    
    def check_repository(self, repo_path: str) -> Dict[str, Any]:
        """Check if path is a git repository and get basic info"""
        repo = GitRepository(repo_path)
        
        result = {
            'is_git_repo': repo.is_git_repo(),
            'git_root': str(repo.git_root) if repo.git_root else None,
            'current_branch': repo.get_current_branch(),
            'current_commit': repo.get_current_commit()
        }
        
        return result
    
    def get_status(self, repo_path: str) -> Dict[str, Any]:
        """Get detailed git status"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {
                'is_git_repo': False,
                'modified_files': [],
                'untracked_files': [],
                'staged_files': []
            }
        
        status = repo.get_status()
        status['is_git_repo'] = True
        
        return status
    
    def get_history(self, repo_path: str, limit: int = 10) -> Dict[str, Any]:
        """Get commit history"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {'commits': []}
        
        commits = repo.get_commit_history(limit)
        return {'commits': commits}
    
    def get_diff(self, repo_path: str, commit_hash: str) -> Dict[str, Any]:
        """Get diff since a commit"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {'changes': []}
        
        changes = repo.get_diff(commit_hash)
        return {'changes': changes}
    
    def get_branches(self, repo_path: str) -> Dict[str, Any]:
        """Get all branches"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {'branches': []}
        
        branches = repo.get_branches()
        return {'branches': branches}
    
    def get_files_needing_reindex(self, repo_path: str, index_path: Optional[str] = None,
                                 code_only: bool = False) -> Dict[str, Any]:
        """Identify files that need re-indexing based on git changes"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {'files_to_reindex': []}
        
        files_to_reindex = []
        
        # Get current status
        status = repo.get_status()
        
        # Add modified and untracked files
        all_changed_files = status['modified_files'] + status['untracked_files']
        
        # If we have an index path, check against last indexed commit
        if index_path:
            index_dir = Path(index_path)
            tracking_file = index_dir / 'last_indexed_commit.txt'
            
            if tracking_file.exists():
                try:
                    with open(tracking_file, 'r') as f:
                        last_commit = f.read().strip()
                    
                    # Get diff since last indexed commit
                    diff_changes = repo.get_diff(last_commit)
                    
                    # Add files changed since last index
                    for change in diff_changes:
                        if change['status'] in ['added', 'modified']:
                            all_changed_files.append(change['file'])
                except Exception as e:
                    self.logger.warning(f"Failed to read tracking file: {e}")
        
        # Filter files
        for filepath in set(all_changed_files):  # Remove duplicates
            filepath_obj = Path(filepath)
            
            # Check if file exists (not deleted)
            if not filepath_obj.exists():
                continue
            
            # Filter by extension if code_only is True
            if code_only:
                ext = filepath_obj.suffix.lstrip('.')
                if ext not in CODE_EXTENSIONS:
                    continue
            
            # Check if ignored by git
            if not repo.is_ignored(filepath):
                files_to_reindex.append(filepath)
        
        return {'files_to_reindex': sorted(files_to_reindex)}
    
    def update_index_tracking(self, repo_path: str, index_path: str) -> Dict[str, Any]:
        """Update index tracking with current commit hash"""
        repo = GitRepository(repo_path)
        
        if not repo.is_git_repo():
            return {'success': False, 'error': 'Not a git repository'}
        
        current_commit = repo.get_current_commit()
        if not current_commit:
            return {'success': False, 'error': 'Could not get current commit'}
        
        try:
            index_dir = Path(index_path)
            index_dir.mkdir(parents=True, exist_ok=True)
            
            tracking_file = index_dir / 'last_indexed_commit.txt'
            with open(tracking_file, 'w') as f:
                f.write(current_commit)
            
            return {
                'success': True,
                'commit_hash': current_commit,
                'tracking_file': str(tracking_file)
            }
        except Exception as e:
            return {'success': False, 'error': str(e)}


def create_argument_parser():
    """Create command line argument parser"""
    parser = argparse.ArgumentParser(
        description="Git Integration and Change Tracking for MCP RAG Indexer",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument('--version', action='version', version=f'Git Tracker v{VERSION}')
    
    # Action commands (mutually exclusive)
    action_group = parser.add_mutually_exclusive_group(required=True)
    action_group.add_argument('--check-repo', type=str, metavar='PATH',
                             help='Check if path is a git repository')
    action_group.add_argument('--status', type=str, metavar='PATH',
                             help='Get git status for repository')
    action_group.add_argument('--history', type=str, metavar='PATH',
                             help='Get commit history')
    action_group.add_argument('--diff', type=str, metavar='COMMIT',
                             help='Get diff since commit')
    action_group.add_argument('--branches', type=str, metavar='PATH',
                             help='List all branches')
    action_group.add_argument('--reindex-needed', type=str, metavar='PATH',
                             help='Get files that need re-indexing')
    action_group.add_argument('--update-tracking', type=str, metavar='PATH',
                             help='Update index tracking with current commit')
    
    # Optional parameters
    parser.add_argument('--target', type=str, metavar='PATH',
                       help='Target directory (for diff command)')
    parser.add_argument('--index-path', type=str, metavar='PATH',
                       help='Path to index directory')
    parser.add_argument('--limit', type=int, default=10, metavar='N',
                       help='Limit number of results (default: 10)')
    parser.add_argument('--code-only', action='store_true',
                       help='Only include code files in results')
    
    # Output format
    parser.add_argument('--format', choices=['json', 'text'], default='json',
                       help='Output format (default: json)')
    
    # Logging
    parser.add_argument('--log-level', choices=['debug', 'info', 'warning', 'error'],
                       default='info', help='Set logging level')
    
    return parser


def format_output_json(data: Dict[str, Any]) -> str:
    """Format output as JSON"""
    return json.dumps(data, indent=2)


def format_output_text(data: Dict[str, Any], action: str) -> str:
    """Format output as plain text"""
    lines = []
    
    if action == 'check-repo':
        lines.append(f"Git Repository Check:")
        lines.append(f"  Is Git Repo: {data.get('is_git_repo', False)}")
        if data.get('git_root'):
            lines.append(f"  Git Root: {data['git_root']}")
        if data.get('current_branch'):
            lines.append(f"  Current Branch: {data['current_branch']}")
        if data.get('current_commit'):
            lines.append(f"  Current Commit: {data['current_commit'][:8]}...")
    
    elif action == 'status':
        lines.append("Status:")
        lines.append(f"  Modified Files: {len(data.get('modified_files', []))}")
        for f in data.get('modified_files', []):
            lines.append(f"    M {f}")
        
        lines.append(f"  Untracked Files: {len(data.get('untracked_files', []))}")
        for f in data.get('untracked_files', []):
            lines.append(f"    ?? {f}")
    
    elif action == 'history':
        lines.append("Commit History:")
        for commit in data.get('commits', []):
            lines.append(f"  {commit['hash'][:8]} {commit['message']}")
            lines.append(f"    Author: {commit['author']}")
            lines.append(f"    Date: {commit['date']}")
            lines.append("")
    
    elif action == 'branches':
        lines.append("Branches:")
        for branch in data.get('branches', []):
            marker = "* " if branch.get('is_current') else "  "
            lines.append(f"{marker}{branch['name']}")
    
    else:
        # Generic format
        lines.append(f"Result: {json.dumps(data, indent=2)}")
    
    return '\n'.join(lines)


def main():
    """Main entry point"""
    parser = create_argument_parser()
    args = parser.parse_args()
    
    try:
        tracker = GitTracker(args.log_level)
        
        # Determine action and execute
        if args.check_repo:
            result = tracker.check_repository(args.check_repo)
            action = 'check-repo'
            
        elif args.status:
            result = tracker.get_status(args.status)
            action = 'status'
            
        elif args.history:
            result = tracker.get_history(args.history, args.limit)
            action = 'history'
            
        elif args.diff:
            target = args.target or '.'
            result = tracker.get_diff(target, args.diff)
            action = 'diff'
            
        elif args.branches:
            result = tracker.get_branches(args.branches)
            action = 'branches'
            
        elif args.reindex_needed:
            result = tracker.get_files_needing_reindex(
                args.reindex_needed,
                args.index_path,
                args.code_only
            )
            action = 'reindex'
            
        elif args.update_tracking:
            if not args.index_path:
                print("Error: --index-path required for --update-tracking", file=sys.stderr)
                return 1
            result = tracker.update_index_tracking(args.update_tracking, args.index_path)
            action = 'update'
            
        else:
            parser.print_help()
            return 1
        
        # Format and output result
        if args.format == 'json':
            output = format_output_json(result)
        else:
            output = format_output_text(result, action)
        
        print(output)
        
        # Return appropriate exit code
        if 'success' in result and not result['success']:
            return 1
        
        return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())