# src/git_manager/cli/ui/progress.py
"""Progress indicators for CLI operations."""

import time
import logging
from typing import Optional, Callable, Any, List, TypeVar
from contextlib import contextmanager
from rich.progress import (
    Progress,
    SpinnerColumn,
    TextColumn,
    BarColumn,
    TaskProgressColumn,
    TimeRemainingColumn,
    TimeElapsedColumn
)
from rich.console import Console
from rich.live import Live
from rich.spinner import Spinner

from .color_schemes import DEFAULT_SCHEME

T = TypeVar('T')


class ProgressManager:
    """Manages progress indicators for long-running operations."""
    
    def __init__(self, console: Optional[Console] = None):
        """Initialize Progress Manager.
        
        Args:
            console: Rich Console instance
        """
        self.console = console or Console()
        self._progress: Optional[Progress] = None
    
    @contextmanager
    def spinner(self, description: str = "Processing..."):
        """Context manager for spinner progress.
        
        Args:
            description: Progress description
            
        Yields:
            Progress task
        """
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=self.console
        ) as progress:
            task = progress.add_task(description, total=None)
            try:
                yield task
            finally:
                progress.update(task, completed=True)
    
    @contextmanager
    def bar(self, description: str = "Processing...", total: int = 100):
        """Context manager for bar progress.
        
        Args:
            description: Progress description
            total: Total items
            
        Yields:
            Tuple of (progress, task_id)
        """
        with Progress(
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TaskProgressColumn(),
            TimeRemainingColumn(),
            TimeElapsedColumn(),
            console=self.console
        ) as progress:
            task = progress.add_task(description, total=total)
            try:
                yield progress, task
            finally:
                progress.update(task, completed=total)
    
    def execute_with_spinner(
        self,
        func: Callable,
        description: str = "Processing...",
        *args,
        **kwargs
    ):
        """Execute function with spinner.
        
        Args:
            func: Function to execute
            description: Progress description
            *args: Function arguments
            **kwargs: Function keyword arguments
            
        Returns:
            Function result
        """
        with self.spinner(description):
            return func(*args, **kwargs)
    
    def execute_with_progress(
        self,
        func: Callable,
        items: list,
        description: str = "Processing items..."
    ):
        """Execute function on items with progress bar.
        
        Args:
            func: Function to execute on each item
            items: List of items to process
            description: Progress description
            
        Returns:
            List of results
        """
        results = []
        with self.bar(description, total=len(items)) as (progress, task):
            for item in items:
                result = func(item)
                results.append(result)
                progress.update(task, advance=1)
        return results
    
    def fetch_repositories(
        self,
        func: Callable,
        account_name: str,
        platform: str,
        *args,
        **kwargs
    ) -> Optional[List[Any]]:
        """
        Execute repository fetch operation with loading indicator.
        
        Specialized for fetching repositories from any platform.
        Shows spinner while fetching and handles errors gracefully.
        
        Args:
            func: Function to fetch repositories
            account_name: Name of the account
            platform: Platform name (github, gitlab, etc.)
            *args: Function arguments
            **kwargs: Function keyword arguments
        
        Returns:
            List of repositories or None on error
        """
        logger = logging.getLogger(__name__)
        
        try:
            scheme = DEFAULT_SCHEME
            description = f"⏳ Fetching repositories for {account_name} ({platform.upper()})"
            with self.spinner(description):
                result = func(*args, **kwargs)
            
            if result:
                self.console.print(f"[{scheme.success}]✓ Found {len(result)} repositories[/{scheme.success}]")
            else:
                self.console.print(f"[{scheme.warning}]⚠️  No repositories found[/{scheme.warning}]")
            
            return result
        
        except Exception as e:
            logger.error(f"Repository fetch failed for {account_name}: {e}")
            self.console.print(f"[{scheme.error}]✗ Failed to fetch repositories: {str(e)}[/{scheme.error}]")
            return None
    
    def fetch_with_retry(
        self,
        func: Callable,
        description: str = "Fetching...",
        max_retries: int = 3,
        retry_delay: float = 1.0,
        *args,
        **kwargs
    ) -> Optional[Any]:
        """
        Execute function with retry logic and spinner.
        
        Useful for network operations that may be flaky.
        
        Args:
            func: Function to execute
            description: Progress description
            max_retries: Maximum number of retries
            retry_delay: Delay between retries in seconds
            *args: Function arguments
            **kwargs: Function keyword arguments
        
        Returns:
            Function result or None on failure
        """
        logger = logging.getLogger(__name__)
        scheme = DEFAULT_SCHEME
        
        for attempt in range(1, max_retries + 1):
            try:
                desc = f"{description} (attempt {attempt}/{max_retries})"
                with self.spinner(desc):
                    return func(*args, **kwargs)
            except Exception as e:
                logger.warning(f"Attempt {attempt} failed: {e}")
                
                if attempt < max_retries:
                    self.console.print(f"[{scheme.warning}]⚠️  Retrying in {retry_delay}s...[/{scheme.warning}]")
                    time.sleep(retry_delay)
                else:
                    self.console.print(f"[{scheme.error}]✗ Failed after {max_retries} attempts[/{scheme.error}]")
                    return None
        
        return None
    
    def execute_batch_with_progress(
        self,
        func: Callable,
        items: List[Any],
        description: str = "Processing...",
        show_item_names: bool = False
    ) -> List[Any]:
        """
        Execute function on batch of items with detailed progress.
        
        Args:
            func: Function to execute on each item
            items: List of items to process
            description: Progress description
            show_item_names: Whether to show item names in progress
        
        Returns:
            List of results
        """
        results = []
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TaskProgressColumn(),
            TimeElapsedColumn(),
            console=self.console
        ) as progress:
            task = progress.add_task(description, total=len(items))
            
            for idx, item in enumerate(items, 1):
                try:
                    # Update description with current item if requested
                    if show_item_names:
                        item_name = str(item)[:30]  # Truncate long names
                        progress.update(
                            task,
                            description=f"{description} [{idx}/{len(items)}] {item_name}"
                        )
                    
                    result = func(item)
                    results.append(result)
                except Exception as e:
                    logging.getLogger(__name__).error(f"Error processing item {idx}: {e}")
                    results.append(None)
                
                progress.update(task, advance=1)
        
        return results


def show_spinner(description: str = "Processing..."):
    """Decorator to show spinner during function execution.
    
    Args:
        description: Progress description
        
    Returns:
        Decorator function
    """
    def decorator(func):
        def wrapper(*args, **kwargs):
            manager = ProgressManager()
            return manager.execute_with_spinner(func, description, *args, **kwargs)
        return wrapper
    return decorator


# # Example usage functions
# def demo_spinner():
#     """Demo spinner usage."""
#     manager = ProgressManager()
    
#     with manager.spinner("Loading accounts..."):
#         time.sleep(2)
    
#     print("Done!")


# def demo_progress_bar():
#     """Demo progress bar usage."""
#     manager = ProgressManager()
    
#     items = range(100)
    
#     def process_item(item):
#         time.sleep(0.05)
#         return item * 2
    
#     results = manager.execute_with_progress(
#         process_item,
#         list(items),
#         "Processing items..."
#     )
    
#     print(f"Processed {len(results)} items")


# if __name__ == '__main__':
#     # Demo
#     demo_spinner()
#     demo_progress_bar()