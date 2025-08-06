import React, { useState, useEffect, useMemo } from 'react';
import './Table.css';

interface TableColumn {
    key: string;
    header: string;
    sortable?: boolean;
    width?: string;
    render?: (value: any, row: any) => React.ReactNode;
}

interface TableProps {
    columns: TableColumn[];
    data: any[];
    onRowClick?: (row: any) => void;
    loading?: boolean;
    pagination?: boolean;
    pageSize?: number;
}

const Table: React.FC<TableProps> = ({
    columns,
    data,
    onRowClick,
    loading = false,
    pagination = true,
    pageSize = 10
}) => {
    const [sortColumn, setSortColumn] = useState<string | null>(null);
    const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
    const [currentPage, setCurrentPage] = useState(1);
    const [searchTerm, setSearchTerm] = useState('');

    // Filter data based on search
    const filteredData = useMemo(() => {
        if (!searchTerm) return data;
        
        return data.filter(row => {
            return Object.values(row).some(value => 
                String(value).toLowerCase().includes(searchTerm.toLowerCase())
            );
        });
    }, [data, searchTerm]);

    // Sort data
    const sortedData = useMemo(() => {
        if (!sortColumn) return filteredData;
        
        return [...filteredData].sort((a, b) => {
            const aValue = a[sortColumn];
            const bValue = b[sortColumn];
            
            if (aValue === bValue) return 0;
            
            const comparison = aValue < bValue ? -1 : 1;
            return sortDirection === 'asc' ? comparison : -comparison;
        });
    }, [filteredData, sortColumn, sortDirection]);

    // Paginate data
    const paginatedData = useMemo(() => {
        if (!pagination) return sortedData;
        
        const startIndex = (currentPage - 1) * pageSize;
        const endIndex = startIndex + pageSize;
        return sortedData.slice(startIndex, endIndex);
    }, [sortedData, currentPage, pageSize, pagination]);

    const totalPages = Math.ceil(sortedData.length / pageSize);

    const handleSort = (columnKey: string) => {
        if (sortColumn === columnKey) {
            setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
        } else {
            setSortColumn(columnKey);
            setSortDirection('asc');
        }
    };

    const handlePageChange = (page: number) => {
        setCurrentPage(Math.max(1, Math.min(page, totalPages)));
    };

    useEffect(() => {
        setCurrentPage(1);
    }, [searchTerm]);

    if (loading) {
        return (
            <div className="table-loading">
                <div className="spinner"></div>
                <p>Loading data...</p>
            </div>
        );
    }

    return (
        <div className="table-container">
            <div className="table-controls">
                <input
                    type="text"
                    placeholder="Search..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="table-search"
                />
            </div>

            <table className="data-table">
                <thead>
                    <tr>
                        {columns.map(column => (
                            <th
                                key={column.key}
                                style={{ width: column.width }}
                                onClick={() => column.sortable && handleSort(column.key)}
                                className={column.sortable ? 'sortable' : ''}
                            >
                                {column.header}
                                {sortColumn === column.key && (
                                    <span className="sort-indicator">
                                        {sortDirection === 'asc' ? '▲' : '▼'}
                                    </span>
                                )}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {paginatedData.map((row, index) => (
                        <tr
                            key={index}
                            onClick={() => onRowClick?.(row)}
                            className={onRowClick ? 'clickable' : ''}
                        >
                            {columns.map(column => (
                                <td key={column.key}>
                                    {column.render 
                                        ? column.render(row[column.key], row)
                                        : row[column.key]}
                                </td>
                            ))}
                        </tr>
                    ))}
                </tbody>
            </table>

            {pagination && totalPages > 1 && (
                <div className="table-pagination">
                    <button
                        onClick={() => handlePageChange(currentPage - 1)}
                        disabled={currentPage === 1}
                    >
                        Previous
                    </button>
                    <span>
                        Page {currentPage} of {totalPages}
                    </span>
                    <button
                        onClick={() => handlePageChange(currentPage + 1)}
                        disabled={currentPage === totalPages}
                    >
                        Next
                    </button>
                </div>
            )}
        </div>
    );
};

export default Table;