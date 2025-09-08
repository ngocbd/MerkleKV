package merklekv

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNew(t *testing.T) {
	client := New("localhost", 7379)
	
	assert.Equal(t, "localhost", client.host)
	assert.Equal(t, 7379, client.port)
	assert.Equal(t, 5*time.Second, client.timeout)
	assert.False(t, client.IsConnected())
}

func TestNewWithTimeout(t *testing.T) {
	timeout := 10 * time.Second
	client := NewWithTimeout("example.com", 9999, timeout)
	
	assert.Equal(t, "example.com", client.host)
	assert.Equal(t, 9999, client.port)
	assert.Equal(t, timeout, client.timeout)
	assert.False(t, client.IsConnected())
}

func TestConnectFailure(t *testing.T) {
	client := New("localhost", 99999) // Wrong port
	
	err := client.Connect()
	
	assert.Error(t, err)
	var connErr *ConnectionError
	assert.True(t, errors.As(err, &connErr))
	assert.Equal(t, "connect", connErr.Op)
	assert.False(t, client.IsConnected())
}

func TestConnectWithContextTimeout(t *testing.T) {
	client := New("1.1.1.1", 99999) // Unreachable address
	
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	
	err := client.ConnectWithContext(ctx)
	
	assert.Error(t, err)
	var connErr *ConnectionError
	assert.True(t, errors.As(err, &connErr))
	assert.False(t, client.IsConnected())
}

func TestClose(t *testing.T) {
	client := New("localhost", 7379)
	
	// Close before connecting should not error
	err := client.Close()
	assert.NoError(t, err)
	assert.False(t, client.IsConnected())
}

func TestGetEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	
	_, err := client.Get("")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestGetWithContextEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	ctx := context.Background()
	
	_, err := client.GetWithContext(ctx, "")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestSetEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	
	err := client.Set("", "value")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestSetWithContextEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	ctx := context.Background()
	
	err := client.SetWithContext(ctx, "", "value")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestDeleteEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	
	err := client.Delete("")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestDeleteWithContextEmptyKey(t *testing.T) {
	client := New("localhost", 7379)
	ctx := context.Background()
	
	err := client.DeleteWithContext(ctx, "")
	assert.Equal(t, ErrEmptyKey, err)
}

func TestOperationsNotConnected(t *testing.T) {
	client := New("localhost", 7379)
	
	// Test all operations fail when not connected
	_, err := client.Get("key")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.Set("key", "value")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.Delete("key")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.Ping()
	assert.Equal(t, ErrNotConnected, err)
}

func TestContextOperationsNotConnected(t *testing.T) {
	client := New("localhost", 7379)
	ctx := context.Background()
	
	// Test all context operations fail when not connected
	_, err := client.GetWithContext(ctx, "key")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.SetWithContext(ctx, "key", "value")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.DeleteWithContext(ctx, "key")
	assert.Equal(t, ErrNotConnected, err)
	
	err = client.PingWithContext(ctx)
	assert.Equal(t, ErrNotConnected, err)
}

func TestConnectionError(t *testing.T) {
	err := &ConnectionError{Op: "test", Err: errors.New("network error")}
	
	assert.Equal(t, "connection error during test: network error", err.Error())
	assert.Equal(t, "network error", err.Unwrap().Error())
}

func TestTimeoutError(t *testing.T) {
	err := &TimeoutError{Op: "test", Timeout: "5s"}
	
	assert.Equal(t, "timeout during test after 5s", err.Error())
}

func TestProtocolError(t *testing.T) {
	err := &ProtocolError{Op: "test", Message: "invalid command"}
	
	assert.Equal(t, "protocol error during test: invalid command", err.Error())
}

// Tests for Pipeline functionality
func TestPipelineEmptyCommands(t *testing.T) {
	client := New("localhost", 7379)
	
	responses, err := client.Pipeline([]string{})
	
	assert.NoError(t, err)
	assert.Empty(t, responses)
}

func TestPipelineNotConnected(t *testing.T) {
	client := New("localhost", 7379)
	
	_, err := client.Pipeline([]string{"GET test"})
	
	assert.Equal(t, ErrNotConnected, err)
}

// Tests for HealthCheck functionality
func TestHealthCheckNotConnected(t *testing.T) {
	client := New("localhost", 7379)
	
	_, err := client.HealthCheck()
	
	assert.Equal(t, ErrNotConnected, err)
}

// Test TCP_NODELAY is applied (coverage for connection setup)
func TestConnectWithTCPNodelay(t *testing.T) {
	client := New("localhost", 7379)
	
	err := client.Connect()
	defer client.Close()
	
	// Connection should succeed with TCP_NODELAY enabled
	assert.NoError(t, err)
	assert.True(t, client.IsConnected())
}

// Test formatSetValue function coverage
func TestFormatSetValue(t *testing.T) {
	// Test empty string formatting
	assert.Equal(t, `""`, formatSetValue(""))
	
	// Test non-empty string
	assert.Equal(t, "test", formatSetValue("test"))
	
	// Test string with spaces
	assert.Equal(t, "hello world", formatSetValue("hello world"))
}
