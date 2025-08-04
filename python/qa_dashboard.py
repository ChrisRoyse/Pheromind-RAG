#!/usr/bin/env python3
"""
Quality Metrics Dashboard and Reporting System
Provides comprehensive quality tracking, trend analysis, and reporting
"""

import json
import time
import sqlite3
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
import statistics
from pathlib import Path
import sys
import os

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from indexer_universal import UniversalCodeIndexer, ValidationQualityAssurance


class QualityMetricsCollector:
    """
    Collects and stores quality metrics for analysis and reporting.
    """
    
    def __init__(self, db_path: str = "quality_metrics.db"):
        self.db_path = db_path
        self._init_database()
        
    def _init_database(self):
        """Initialize SQLite database for metrics storage."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Create metrics table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS quality_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                session_id TEXT,
                total_processed INTEGER,
                documentation_found INTEGER,
                documentation_coverage REAL,
                validation_success_rate REAL,
                avg_confidence REAL,
                high_confidence_count INTEGER,
                low_confidence_count INTEGER,
                edge_cases_handled INTEGER,
                avg_processing_time REAL,
                processing_time_std REAL,
                validation_failures INTEGER,
                recent_error_count INTEGER
            )
        ''')
        
        # Create validation results table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS validation_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                session_id TEXT,
                file_path TEXT,
                language TEXT,
                validation_passed BOOLEAN,
                quality_score REAL,
                warning_count INTEGER,
                error_count INTEGER,
                checks_performed TEXT,
                processing_time REAL
            )
        ''')
        
        # Create system health table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS system_health (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT,
                alert_count INTEGER,
                alerts TEXT,
                recommendations TEXT,
                avg_processing_time REAL,
                validation_error_rate REAL,
                high_confidence_ratio REAL,
                recent_error_count INTEGER
            )
        ''')
        
        conn.commit()
        conn.close()
    
    def record_quality_metrics(self, qa_system: ValidationQualityAssurance, session_id: str = None):
        """Record current quality metrics from QA system."""
        if session_id is None:
            session_id = f"session_{int(time.time())}"
            
        report = qa_system.generate_quality_report()
        
        if report.get('status') == 'insufficient_data':
            return
            
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        summary = report.get('summary', {})
        performance = report.get('performance', {})
        quality_indicators = report.get('quality_indicators', {})
        
        cursor.execute('''
            INSERT INTO quality_metrics (
                session_id, total_processed, documentation_found, documentation_coverage,
                validation_success_rate, high_confidence_count, low_confidence_count,
                edge_cases_handled, avg_processing_time, processing_time_std,
                validation_failures, recent_error_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            session_id,
            summary.get('total_processed', 0),
            summary.get('documentation_found', 0),
            summary.get('documentation_coverage', 0.0),
            summary.get('validation_success_rate', 1.0),
            qa_system.detection_stats.get('high_confidence_detections', 0),
            qa_system.detection_stats.get('low_confidence_detections', 0),
            summary.get('edge_cases_handled', 0),
            performance.get('avg_processing_time', 0.0),
            performance.get('processing_time_std', 0.0),
            qa_system.detection_stats.get('validation_failures', 0),
            quality_indicators.get('recent_error_count', 0)
        ))
        
        conn.commit()
        conn.close()
    
    def record_validation_result(self, validation_result: Dict, file_path: str, language: str, session_id: str = None):
        """Record individual validation result."""
        if session_id is None:
            session_id = f"session_{int(time.time())}"
            
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            INSERT INTO validation_results (
                session_id, file_path, language, validation_passed, quality_score,
                warning_count, error_count, checks_performed, processing_time
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            session_id,
            file_path,
            language,
            validation_result.get('passed', False),
            validation_result.get('quality_score', 0.0),
            len(validation_result.get('warnings', [])),
            len(validation_result.get('errors', [])),
            json.dumps(validation_result.get('checks_performed', [])),
            validation_result.get('validation_time', 0.0)
        ))
        
        conn.commit()
        conn.close()
    
    def record_system_health(self, health_report: Dict, session_id: str = None):
        """Record system health status."""
        if session_id is None:
            session_id = f"session_{int(time.time())}"
            
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            INSERT INTO system_health (
                status, alert_count, alerts, recommendations,
                avg_processing_time, validation_error_rate, high_confidence_ratio,
                recent_error_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            health_report.get('status', 'unknown'),
            len(health_report.get('alerts', [])),
            json.dumps(health_report.get('alerts', [])),
            json.dumps(health_report.get('recommendations', [])),
            health_report.get('metrics', {}).get('avg_processing_time', 0.0),
            health_report.get('metrics', {}).get('validation_error_rate', 0.0),
            health_report.get('metrics', {}).get('high_confidence_ratio', 0.0),
            health_report.get('metrics', {}).get('recent_error_count', 0)
        ))
        
        conn.commit()
        conn.close()
    
    def get_quality_trends(self, days: int = 7) -> Dict[str, Any]:
        """Get quality trends over specified time period."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # Get metrics from last N days
        since_date = datetime.now() - timedelta(days=days)
        
        cursor.execute('''
            SELECT 
                documentation_coverage,
                validation_success_rate,
                avg_processing_time,
                recent_error_count,
                timestamp
            FROM quality_metrics 
            WHERE timestamp >= ?
            ORDER BY timestamp
        ''', (since_date,))
        
        rows = cursor.fetchall()
        conn.close()
        
        if not rows:
            return {'status': 'no_data', 'message': f'No data available for last {days} days'}
        
        # Calculate trends
        coverages = [row[0] for row in rows if row[0] is not None]
        success_rates = [row[1] for row in rows if row[1] is not None]
        processing_times = [row[2] for row in rows if row[2] is not None and row[2] > 0]
        error_counts = [row[3] for row in rows if row[3] is not None]
        
        trends = {
            'period_days': days,
            'data_points': len(rows),
            'documentation_coverage': {
                'current': coverages[-1] if coverages else 0,
                'average': statistics.mean(coverages) if coverages else 0,
                'trend': self._calculate_trend(coverages) if len(coverages) >= 2 else 'stable'
            },
            'validation_success_rate': {
                'current': success_rates[-1] if success_rates else 0,
                'average': statistics.mean(success_rates) if success_rates else 0,
                'trend': self._calculate_trend(success_rates) if len(success_rates) >= 2 else 'stable'
            },
            'processing_performance': {
                'current': processing_times[-1] if processing_times else 0,
                'average': statistics.mean(processing_times) if processing_times else 0,
                'trend': self._calculate_trend(processing_times, reverse=True) if len(processing_times) >= 2 else 'stable'
            },
            'error_frequency': {
                'current': error_counts[-1] if error_counts else 0,
                'average': statistics.mean(error_counts) if error_counts else 0,
                'trend': self._calculate_trend(error_counts, reverse=True) if len(error_counts) >= 2 else 'stable'
            }
        }
        
        return trends
    
    def _calculate_trend(self, values: List[float], reverse: bool = False) -> str:
        """Calculate trend direction from list of values."""
        if len(values) < 2:
            return 'stable'
            
        # Simple linear trend calculation
        first_half = values[:len(values)//2]
        second_half = values[len(values)//2:]
        
        first_avg = statistics.mean(first_half)
        second_avg = statistics.mean(second_half)
        
        change_threshold = 0.05  # 5% change threshold
        
        if reverse:
            # For performance metrics, lower is better
            if second_avg < first_avg * (1 - change_threshold):
                return 'improving'
            elif second_avg > first_avg * (1 + change_threshold):
                return 'declining'
        else:
            # For quality metrics, higher is better
            if second_avg > first_avg * (1 + change_threshold):
                return 'improving'
            elif second_avg < first_avg * (1 - change_threshold):
                return 'declining'
                
        return 'stable'
    
    def get_system_health_history(self, hours: int = 24) -> Dict[str, Any]:
        """Get system health history over specified time period."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        since_time = datetime.now() - timedelta(hours=hours)
        
        cursor.execute('''
            SELECT status, alert_count, timestamp
            FROM system_health 
            WHERE timestamp >= ?
            ORDER BY timestamp
        ''', (since_time,))
        
        rows = cursor.fetchall()
        conn.close()
        
        if not rows:
            return {'status': 'no_data', 'message': f'No health data available for last {hours} hours'}
        
        # Analyze health status distribution
        status_counts = {}
        alert_counts = []
        
        for row in rows:
            status = row[0]
            alert_count = row[1] or 0
            
            status_counts[status] = status_counts.get(status, 0) + 1
            alert_counts.append(alert_count)
        
        total_checks = len(rows)
        
        return {
            'period_hours': hours,
            'total_health_checks': total_checks,
            'status_distribution': {
                status: {'count': count, 'percentage': (count / total_checks) * 100}
                for status, count in status_counts.items()
            },
            'alert_statistics': {
                'total_alerts': sum(alert_counts),
                'average_alerts_per_check': statistics.mean(alert_counts) if alert_counts else 0,
                'max_alerts_in_period': max(alert_counts) if alert_counts else 0
            },
            'overall_health_score': self._calculate_health_score(status_counts, total_checks)
        }
    
    def _calculate_health_score(self, status_counts: Dict[str, int], total_checks: int) -> float:
        """Calculate overall health score from status distribution."""
        if total_checks == 0:
            return 0.0
            
        # Weight different statuses
        status_weights = {
            'healthy': 1.0,
            'degraded': 0.6,
            'critical': 0.0
        }
        
        weighted_score = 0.0
        for status, count in status_counts.items():
            weight = status_weights.get(status, 0.5)  # Default weight for unknown statuses
            weighted_score += (count / total_checks) * weight
            
        return weighted_score


class QualityDashboard:
    """
    Comprehensive quality dashboard for monitoring and reporting.
    """
    
    def __init__(self, db_path: str = "quality_metrics.db"):
        self.collector = QualityMetricsCollector(db_path)
        
    def generate_dashboard_report(self) -> Dict[str, Any]:
        """Generate comprehensive dashboard report."""
        current_time = datetime.now()
        
        # Get quality trends for different periods
        daily_trends = self.collector.get_quality_trends(days=1)
        weekly_trends = self.collector.get_quality_trends(days=7)
        monthly_trends = self.collector.get_quality_trends(days=30)
        
        # Get system health history
        hourly_health = self.collector.get_system_health_history(hours=1)
        daily_health = self.collector.get_system_health_history(hours=24)
        
        dashboard = {
            'generated_at': current_time.isoformat(),
            'summary': {
                'overall_status': self._determine_overall_status(daily_trends, daily_health),
                'key_metrics': self._extract_key_metrics(daily_trends),
                'health_score': daily_health.get('overall_health_score', 0.0),
                'alerts_active': self._count_active_alerts(hourly_health)
            },
            'quality_trends': {
                'last_24_hours': daily_trends,
                'last_7_days': weekly_trends,
                'last_30_days': monthly_trends
            },
            'system_health': {
                'last_hour': hourly_health,
                'last_24_hours': daily_health
            },
            'recommendations': self._generate_recommendations(daily_trends, daily_health)
        }
        
        return dashboard
    
    def _determine_overall_status(self, trends: Dict, health: Dict) -> str:
        """Determine overall system status."""
        health_score = health.get('overall_health_score', 0.0)
        
        if health_score >= 0.9:
            return 'excellent'
        elif health_score >= 0.7:
            return 'good'
        elif health_score >= 0.5:
            return 'fair'
        else:
            return 'poor'
    
    def _extract_key_metrics(self, trends: Dict) -> Dict[str, Any]:
        """Extract key metrics for dashboard summary."""
        if trends.get('status') == 'no_data':
            return {}
            
        return {
            'documentation_coverage': trends.get('documentation_coverage', {}).get('current', 0.0),
            'validation_success_rate': trends.get('validation_success_rate', {}).get('current', 0.0),
            'avg_processing_time': trends.get('processing_performance', {}).get('current', 0.0),
            'error_frequency': trends.get('error_frequency', {}).get('current', 0)
        }
    
    def _count_active_alerts(self, health: Dict) -> int:
        """Count active alerts from recent health data."""
        return health.get('alert_statistics', {}).get('total_alerts', 0)
    
    def _generate_recommendations(self, trends: Dict, health: Dict) -> List[str]:
        """Generate actionable recommendations based on trends and health."""
        recommendations = []
        
        # Check documentation coverage trend
        if trends.get('status') != 'no_data':
            doc_trend = trends.get('documentation_coverage', {}).get('trend', 'stable')
            if doc_trend == 'declining':
                recommendations.append("Documentation coverage is declining - consider reviewing documentation standards")
            
            # Check validation success rate
            validation_trend = trends.get('validation_success_rate', {}).get('trend', 'stable')
            if validation_trend == 'declining':
                recommendations.append("Validation success rate is declining - investigate validation failures")
            
            # Check performance trend
            perf_trend = trends.get('processing_performance', {}).get('trend', 'stable')
            if perf_trend == 'declining':
                recommendations.append("Processing performance is declining - consider optimization")
        
        # Check health score
        health_score = health.get('overall_health_score', 1.0)
        if health_score < 0.7:
            recommendations.append("System health score is below threshold - review system alerts")
        
        # Check alert frequency
        alert_count = health.get('alert_statistics', {}).get('total_alerts', 0)
        if alert_count > 5:
            recommendations.append(f"High alert frequency ({alert_count} alerts) - investigate root causes")
        
        if not recommendations:
            recommendations.append("System operating within normal parameters")
            
        return recommendations
    
    def export_dashboard_html(self, output_path: str = "quality_dashboard.html"):
        """Export dashboard as HTML report."""
        dashboard = self.generate_dashboard_report()
        
        html_content = self._generate_html_report(dashboard)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
            
        return output_path
    
    def _generate_html_report(self, dashboard: Dict) -> str:
        """Generate HTML content for dashboard report."""
        summary = dashboard.get('summary', {})
        trends = dashboard.get('quality_trends', {})
        health = dashboard.get('system_health', {})
        recommendations = dashboard.get('recommendations', [])
        
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>MCP RAG Indexer - Quality Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: #2c3e50; color: white; padding: 20px; border-radius: 10px; }}
        .summary {{ background: white; padding: 20px; margin: 20px 0; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .metric-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin: 20px 0; }}
        .metric-card {{ background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .metric-value {{ font-size: 2em; font-weight: bold; color: #3498db; }}
        .metric-label {{ color: #7f8c8d; font-size: 0.9em; }}
        .status-excellent {{ color: #27ae60; }}
        .status-good {{ color: #f39c12; }}
        .status-fair {{ color: #e67e22; }}
        .status-poor {{ color: #e74c3c; }}
        .recommendations {{ background: #ecf0f1; padding: 20px; margin: 20px 0; border-radius: 10px; }}
        .recommendation {{ background: white; padding: 10px; margin: 10px 0; border-left: 4px solid #3498db; }}
        .trend-improving {{ color: #27ae60; }}
        .trend-declining {{ color: #e74c3c; }}
        .trend-stable {{ color: #7f8c8d; }}
        pre {{ background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>MCP RAG Indexer - Quality Dashboard</h1>
            <p>Generated: {dashboard.get('generated_at', 'Unknown')}</p>
        </div>
        
        <div class="summary">
            <h2>System Overview</h2>
            <div class="metric-grid">
                <div class="metric-card">
                    <div class="metric-value status-{summary.get('overall_status', 'unknown')}">{summary.get('overall_status', 'Unknown').title()}</div>
                    <div class="metric-label">Overall Status</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{summary.get('health_score', 0.0):.1%}</div>
                    <div class="metric-label">Health Score</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{summary.get('alerts_active', 0)}</div>
                    <div class="metric-label">Active Alerts</div>
                </div>
            </div>
        </div>
"""
        
        # Add key metrics if available
        key_metrics = summary.get('key_metrics', {})
        if key_metrics:
            html += f"""
        <div class="summary">
            <h2>Key Quality Metrics</h2>
            <div class="metric-grid">
                <div class="metric-card">
                    <div class="metric-value">{key_metrics.get('documentation_coverage', 0.0):.1%}</div>
                    <div class="metric-label">Documentation Coverage</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{key_metrics.get('validation_success_rate', 0.0):.1%}</div>
                    <div class="metric-label">Validation Success Rate</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{key_metrics.get('avg_processing_time', 0.0):.3f}s</div>
                    <div class="metric-label">Avg Processing Time</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{key_metrics.get('error_frequency', 0)}</div>
                    <div class="metric-label">Recent Errors</div>
                </div>
            </div>
        </div>
"""
        
        # Add recommendations
        html += f"""
        <div class="recommendations">
            <h2>Recommendations</h2>
"""
        for rec in recommendations:
            html += f'            <div class="recommendation">{rec}</div>\n'
            
        html += """
        </div>
        
        <div class="summary">
            <h2>Detailed Metrics</h2>
            <h3>Quality Trends (Last 24 Hours)</h3>
            <pre>{}</pre>
            
            <h3>System Health (Last 24 Hours)</h3>
            <pre>{}</pre>
        </div>
    </div>
</body>
</html>
""".format(
            json.dumps(trends.get('last_24_hours', {}), indent=2),
            json.dumps(health.get('last_24_hours', {}), indent=2)
        )
        
        return html


def main():
    """Main entry point for dashboard generation."""
    import argparse
    
    parser = argparse.ArgumentParser(description="MCP RAG Indexer Quality Dashboard")
    parser.add_argument('--action', choices=['report', 'html', 'trends'], 
                       default='report', help='Action to perform')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--days', type=int, default=7, help='Number of days for trends')
    
    args = parser.parse_args()
    
    dashboard = QualityDashboard()
    
    if args.action == 'report':
        report = dashboard.generate_dashboard_report()
        print(json.dumps(report, indent=2))
        
    elif args.action == 'html':
        output_path = args.output or 'quality_dashboard.html'
        generated_path = dashboard.export_dashboard_html(output_path)
        print(f"Dashboard exported to: {generated_path}")
        
    elif args.action == 'trends':
        trends = dashboard.collector.get_quality_trends(args.days)
        print(json.dumps(trends, indent=2))


if __name__ == '__main__':
    main()