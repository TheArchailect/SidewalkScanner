import React, { useEffect, useState } from 'react';


type Props = {
    allFileLoadProgress: Record<string, number>;
};

const LoadingPanel: React.FC<Props> = ({ allFileLoadProgress}) => {

    return (
        <div style={styles.overlay}>
            <div style={styles.box}>
                <h2>Loading data</h2>
                <p>Depending on your connection, this may take multiple minutes</p>
                {Object.entries(allFileLoadProgress).map(([key, value]) => (
                    <div key={key}>
                        <strong>{key}</strong>: {value === 0 ? ("loading...") : ("done")}
                    </div>
                ))}
            </div>
        </div>
    );
};

const styles: { [key: string]: React.CSSProperties } = {
    overlay: {
        position: 'fixed',
        top: 0, left: 0, right: 0, bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.6)',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        zIndex: 9999,
    },
    box: {
        backgroundColor: '#fff',
        padding: '2rem',
        borderRadius: '8px',
        minWidth: '300px',
        textAlign: 'left',
    },
    line: {
        display: 'flex',
        justifyContent: 'space-between',
        marginTop: '0.5rem',
        fontSize: '1rem',
    },
};

export default LoadingPanel;