"""Repository API endpoints."""

from typing import Dict, List, Optional
from ..workflow import CloneWorkflow


class RepositoryAPI:
    """API for repository operations."""
    
    def __init__(self, account_manager, config_manager=None):
        self.workflow = CloneWorkflow(account_manager, config_manager)
        self.account_manager = account_manager
    
    def list_personal_repositories(
        self,
        platform: str,
        account_name: str,
        force_refresh: bool = False
    ) -> Dict:
        """
        List personal repositories.
        
        Args:
            platform: Platform ID ('github', 'gitlab', 'bitbucket')
            account_name: Account name
            force_refresh: Ignore cache
        
        Returns:
            List of repositories
        """
        
        try:
            repositories = self.workflow.fetch_personal_repositories(
                platform,
                account_name,
                force_refresh=force_refresh
            )
            
            return {
                'success': True,
                'platform': platform,
                'account': account_name,
                'count': len(repositories),
                'repositories': [
                    {
                        'id': repo.id,
                        'name': repo.name,
                        'full_name': repo.full_name,
                        'owner': repo.owner,
                        'description': repo.description,
                        'visibility': repo.visibility,
                        'language': repo.language,
                        'size_kb': repo.size_kb,
                        'stars': repo.stars,
                        'updated_at': repo.updated_at,
                        'ssh_url': repo.ssh_url,
                        'https_url': repo.https_url,
                        'web_url': repo.web_url,
                        'is_fork': repo.is_fork,
                        'default_branch': repo.default_branch
                    }
                    for repo in repositories
                ]
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_repository_info(
        self,
        repo_url: str
    ) -> Dict:
        """
        Get repository information from URL.
        
        Args:
            repo_url: Repository URL
        
        Returns:
            Repository information
        """
        
        try:
            analysis = self.workflow.analyze_external_repository(repo_url)
            return analysis
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def search_repositories(
        self,
        platform: str,
        account_name: str,
        query: str,
        force_refresh: bool = False
    ) -> Dict:
        """
        Search repositories.
        
        Args:
            platform: Platform ID
            account_name: Account name
            query: Search query
            force_refresh: Ignore cache
        
        Returns:
            Search results
        """
        
        try:
            repositories = self.workflow.fetch_personal_repositories(
                platform,
                account_name,
                force_refresh=force_refresh
            )
            
            # Filter repositories
            query_lower = query.lower()
            filtered = [
                repo for repo in repositories
                if query_lower in repo.name.lower() or
                   query_lower in (repo.description or '').lower()
            ]
            
            return {
                'success': True,
                'query': query,
                'total': len(repositories),
                'results': len(filtered),
                'repositories': [
                    {
                        'id': repo.id,
                        'name': repo.name,
                        'full_name': repo.full_name,
                        'description': repo.description,
                        'visibility': repo.visibility,
                        'language': repo.language,
                        'stars': repo.stars,
                        'updated_at': repo.updated_at,
                        'web_url': repo.web_url
                    }
                    for repo in filtered
                ]
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def filter_repositories(
        self,
        platform: str,
        account_name: str,
        visibility: Optional[str] = None,
        language: Optional[str] = None,
        is_fork: Optional[bool] = None,
        force_refresh: bool = False
    ) -> Dict:
        """
        Filter repositories by criteria.
        
        Args:
            platform: Platform ID
            account_name: Account name
            visibility: 'private' or 'public'
            language: Programming language
            is_fork: Filter by fork status
            force_refresh: Ignore cache
        
        Returns:
            Filtered repositories
        """
        
        try:
            repositories = self.workflow.fetch_personal_repositories(
                platform,
                account_name,
                force_refresh=force_refresh
            )
            
            # Apply filters
            filtered = repositories
            
            if visibility:
                filtered = [r for r in filtered if r.visibility == visibility]
            
            if language:
                filtered = [r for r in filtered if r.language and language.lower() in r.language.lower()]
            
            if is_fork is not None:
                filtered = [r for r in filtered if r.is_fork == is_fork]
            
            return {
                'success': True,
                'filters': {
                    'visibility': visibility,
                    'language': language,
                    'is_fork': is_fork
                },
                'total': len(repositories),
                'results': len(filtered),
                'repositories': [
                    {
                        'id': repo.id,
                        'name': repo.name,
                        'full_name': repo.full_name,
                        'visibility': repo.visibility,
                        'language': repo.language,
                        'is_fork': repo.is_fork,
                        'web_url': repo.web_url
                    }
                    for repo in filtered
                ]
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
